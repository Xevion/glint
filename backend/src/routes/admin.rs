use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use chrono::Utc;
use serde::Deserialize;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{
        CompleteWorldUploadRequest, CreateJobRequest, CreateSceneRequest, CreateShaderRequest,
        CreateShaderVersionRequest, CreateWorldRequest, CreateWorldUploadResponse, Job, Scene,
        Shader, ShaderVersion, UpdateSceneRequest, World,
    },
    repo::{
        CaptureRepo, JobRepo, PendingUploadRepo, SceneRepo, ShaderRepo, ShaderVersionRepo,
        WorldRepo,
    },
    state::AppState,
};

/// Presigned URL expiry time (5 minutes)
const PRESIGN_EXPIRY_SECS: u64 = 300;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shaders", post(create_shader))
        .route("/shaders/{id}/versions", post(create_shader_version))
        .route("/worlds", get(list_worlds).post(create_world_upload))
        .route("/worlds/{slug}/complete", post(complete_world_upload))
        .route("/scenes", post(create_scene))
        .route("/scenes/{slug}", put(update_scene).delete(disable_scene))
        .route("/jobs", get(list_jobs).post(create_job))
        .route("/jobs/{id}", delete(delete_job))
        .route("/jobs/{id}/cancel", put(cancel_job))
        .route("/jobs/{id}/retry", put(retry_job))
        .route("/jobs/{id}/release", put(release_job))
}

/// POST /api/admin/shaders
async fn create_shader(
    State(state): State<AppState>,
    Json(request): Json<CreateShaderRequest>,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let id = Uuid::new_v4().to_string();
    let shader = ShaderRepo::create(state.db(), &id, &request).await?;
    info!(shader_id = %shader.id, slug = %shader.slug, "Shader created");
    Ok((StatusCode::CREATED, Json(shader)))
}

/// POST /api/admin/shaders/{id}/versions
async fn create_shader_version(
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
    Json(request): Json<CreateShaderVersionRequest>,
) -> AppResult<(StatusCode, Json<ShaderVersion>)> {
    // Verify shader exists
    if !ShaderRepo::exists_by_id(state.db(), &shader_id).await? {
        return Err(AppError::NotFound("Shader not found".into()));
    }

    let id = Uuid::new_v4().to_string();
    let version = ShaderVersionRepo::create(state.db(), &id, &shader_id, &request).await?;

    Ok((StatusCode::CREATED, Json(version)))
}

/// GET /api/admin/worlds
async fn list_worlds(State(state): State<AppState>) -> AppResult<Json<Vec<World>>> {
    let worlds = WorldRepo::list(state.db()).await?;
    Ok(Json(worlds))
}

/// POST /api/admin/worlds
/// Initiates a world upload by returning a presigned URL
async fn create_world_upload(
    State(state): State<AppState>,
    Json(request): Json<crate::models::CreateWorldUploadRequest>,
) -> AppResult<(StatusCode, Json<CreateWorldUploadResponse>)> {
    debug!(slug = %request.slug, "Preparing world upload");

    // Validate hash format (must have algorithm prefix)
    if !request.file_hash.starts_with("sha256:") {
        return Err(AppError::BadRequest(
            "file_hash must have algorithm prefix (e.g., 'sha256:abc123...')".into(),
        ));
    }

    // Validate file size (max 512 MiB)
    const MAX_UPLOAD_SIZE: i64 = 512 * 1024 * 1024;
    if request.file_size_bytes > MAX_UPLOAD_SIZE {
        return Err(AppError::BadRequest(format!(
            "File too large: {} bytes (max {} MiB)",
            request.file_size_bytes,
            MAX_UPLOAD_SIZE / 1024 / 1024
        )));
    }

    // Check if world with this slug already exists
    if WorldRepo::exists_by_slug(state.db(), &request.slug).await? {
        return Err(AppError::Conflict(format!(
            "World with slug '{}' already exists",
            request.slug
        )));
    }

    // Require R2/S3 to be configured
    let s3 = state
        .s3()
        .ok_or_else(|| AppError::ServiceUnavailable("R2/S3 storage not configured".into()))?;

    let r2_config = &state.config().r2;
    let bucket = r2_config.bucket.as_deref().unwrap_or("glint");

    // Generate upload ID and key
    let upload_id = Uuid::new_v4().to_string();
    let upload_key = format!("_uploads/{}.zip", upload_id);
    let expires_at = Utc::now() + chrono::Duration::seconds(PRESIGN_EXPIRY_SECS as i64);

    // Generate presigned PUT URL with hash metadata requirement
    let presigned = s3
        .put_object()
        .bucket(bucket)
        .key(&upload_key)
        .content_type("application/zip")
        .metadata("sha256", request.file_hash.strip_prefix("sha256:").unwrap())
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::builder()
                .expires_in(std::time::Duration::from_secs(PRESIGN_EXPIRY_SECS))
                .build()
                .expect("valid presigning config"),
        )
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to generate presigned URL");
            AppError::Internal(anyhow::anyhow!("Failed to generate presigned URL: {}", e))
        })?;

    // Store pending upload record
    PendingUploadRepo::create(
        state.db(),
        &upload_id,
        &request.slug,
        &request.name,
        request.description.as_deref(),
        &request.minecraft_version,
        &request.file_hash,
        request.file_size_bytes,
        &upload_key,
        expires_at,
    )
    .await?;

    debug!(
        upload_id = %upload_id,
        slug = %request.slug,
        expires_at = %expires_at,
        "Pending upload created"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateWorldUploadResponse {
            upload_id,
            presigned_url: presigned.uri().to_string(),
            expires_at,
        }),
    ))
}

/// POST /api/admin/worlds/{slug}/complete
/// Completes a world upload after file has been uploaded to R2
async fn complete_world_upload(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<CompleteWorldUploadRequest>,
) -> AppResult<(StatusCode, Json<World>)> {
    debug!(slug = %slug, upload_id = %request.upload_id, "Completing world upload");

    let pending = PendingUploadRepo::get_by_id(state.db(), &request.upload_id).await?;

    // Verify slug matches
    if pending.slug != slug {
        return Err(AppError::BadRequest(format!(
            "Upload ID '{}' does not match slug '{}'",
            request.upload_id, slug
        )));
    }

    // Check if expired
    if pending.expires_at < Utc::now() {
        // Clean up expired record
        PendingUploadRepo::delete(state.db(), &request.upload_id).await?;
        return Err(AppError::Gone(
            "Upload has expired. Please start a new upload.".into(),
        ));
    }

    // Require R2/S3
    let s3 = state
        .s3()
        .ok_or_else(|| AppError::ServiceUnavailable("R2/S3 storage not configured".into()))?;

    let r2_config = &state.config().r2;
    let bucket = r2_config.bucket.as_deref().unwrap_or("glint");

    // Verify file exists in R2 via head_object
    let head_result = s3
        .head_object()
        .bucket(bucket)
        .key(&pending.upload_key)
        .send()
        .await;

    let head = match head_result {
        Ok(h) => h,
        Err(e) => {
            warn!(upload_id = %request.upload_id, error = %e, "Upload file not found in R2");
            return Err(AppError::NotFound(
                "Uploaded file not found in storage. Please retry the upload.".into(),
            ));
        }
    };

    // Verify hash from metadata
    let stored_hash = head.metadata().and_then(|m| m.get("sha256").cloned());
    let expected_hash = pending
        .file_hash
        .strip_prefix("sha256:")
        .unwrap_or(&pending.file_hash);

    if stored_hash.as_deref() != Some(expected_hash) {
        warn!(
            upload_id = %request.upload_id,
            expected = %expected_hash,
            actual = ?stored_hash,
            "Upload hash mismatch"
        );
        return Err(AppError::BadRequest(
            "File hash mismatch. The uploaded file may be corrupted.".into(),
        ));
    }

    // Move file to final location: _uploads/{uuid}.zip -> worlds/{slug}.zip
    let final_key = format!("worlds/{}.zip", pending.slug);

    // Copy to final location
    let copy_source = format!("{}/{}", bucket, pending.upload_key);
    s3.copy_object()
        .bucket(bucket)
        .copy_source(&copy_source)
        .key(&final_key)
        .metadata_directive(aws_sdk_s3::types::MetadataDirective::Copy)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to copy upload to final location");
            AppError::Internal(anyhow::anyhow!("Failed to finalize upload: {}", e))
        })?;

    // Delete original upload file
    if let Err(e) = s3
        .delete_object()
        .bucket(bucket)
        .key(&pending.upload_key)
        .send()
        .await
    {
        // Log but don't fail - cleanup will handle it
        warn!(key = %pending.upload_key, error = %e, "Failed to delete temp upload file");
    }

    // Generate public URL
    let file_url = if let Some(ref public_url_prefix) = r2_config.public_url {
        format!("{}/{}", public_url_prefix, final_key)
    } else {
        format!("https://{}.r2.cloudflarestorage.com/{}", bucket, final_key)
    };

    // Create world record
    let world_id = Uuid::new_v4().to_string();
    let world_result = WorldRepo::create(
        state.db(),
        &world_id,
        &CreateWorldRequest {
            name: &pending.name,
            slug: &pending.slug,
            description: pending.description.as_deref(),
            minecraft_version: &pending.minecraft_version,
            file_url: &file_url,
            file_hash: &pending.file_hash,
            size_bytes: pending.size_bytes,
        },
    )
    .await;

    // Handle conflict (another upload completed first)
    if let Err(AppError::Conflict(_)) = &world_result {
        // Clean up the file we just copied
        if let Err(e) = s3
            .delete_object()
            .bucket(bucket)
            .key(&final_key)
            .send()
            .await
        {
            warn!(key = %final_key, error = %e, "Failed to clean up conflicting upload");
        }

        // Delete pending record
        PendingUploadRepo::delete(state.db(), &request.upload_id).await?;

        return Err(AppError::Conflict(format!(
            "World with slug '{}' was created by another upload",
            slug
        )));
    }

    let world = world_result?;

    // Delete pending upload record
    PendingUploadRepo::delete(state.db(), &request.upload_id).await?;

    info!(world_id = %world.id, slug = %world.slug, "World created");
    Ok((StatusCode::CREATED, Json(world)))
}

/// POST /api/admin/scenes
async fn create_scene(
    State(state): State<AppState>,
    Json(request): Json<CreateSceneRequest>,
) -> AppResult<(StatusCode, Json<Scene>)> {
    // Verify world exists
    if !WorldRepo::exists_by_id(state.db(), &request.world_id).await? {
        return Err(AppError::NotFound("World not found".into()));
    }

    // Check world-scoped slug uniqueness (only active scenes)
    if SceneRepo::exists_by_slug_in_world(state.db(), &request.slug, &request.world_id).await? {
        return Err(AppError::Conflict(format!(
            "Scene with slug '{}' already exists in this world",
            request.slug
        )));
    }

    let id = Uuid::new_v4().to_string();
    let scene = SceneRepo::create(state.db(), &id, &request).await?;

    Ok((StatusCode::CREATED, Json(scene)))
}

/// PUT /api/admin/scenes/{slug}
async fn update_scene(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateSceneRequest>,
) -> AppResult<Json<Scene>> {
    // Find scene (world-scoped, active only)
    let scene = SceneRepo::get_by_slug_and_world(state.db(), &slug, &request.world_id).await?;

    // Update scene
    let updated = SceneRepo::update(state.db(), &scene.id, &request).await?;

    // Mark captures as outdated
    CaptureRepo::mark_outdated_for_scene(state.db(), &scene.id).await?;

    Ok(Json(updated))
}

/// DELETE /api/admin/scenes/{slug}
async fn disable_scene(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<WorldIdParam>,
) -> AppResult<StatusCode> {
    let disabled = SceneRepo::disable(state.db(), &slug, &params.world_id).await?;

    if !disabled {
        return Err(AppError::NotFound(format!("Scene '{}' not found", slug)));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/admin/jobs
async fn list_jobs(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<crate::repo::job::JobWithDetails>>> {
    let jobs = JobRepo::list_with_details(state.db()).await?;
    Ok(Json(jobs))
}

/// POST /api/admin/jobs
async fn create_job(
    State(state): State<AppState>,
    Json(request): Json<CreateJobRequest>,
) -> AppResult<(StatusCode, Json<Job>)> {
    // Verify shader version exists
    if !ShaderVersionRepo::exists_by_id(state.db(), &request.shader_version_id).await? {
        return Err(AppError::NotFound("Shader version not found".into()));
    }

    let id = Uuid::new_v4().to_string();
    let job = JobRepo::create(state.db(), &id, &request).await?;

    Ok((StatusCode::CREATED, Json(job)))
}

#[derive(Deserialize)]
struct WorldIdParam {
    world_id: String,
}

/// DELETE /api/admin/jobs/{id}
async fn delete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = JobRepo::delete(state.db(), &job_id).await?;

    if !deleted {
        return Err(AppError::NotFound("Job not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/admin/jobs/{id}/cancel
async fn cancel_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    let cancelled = JobRepo::cancel(state.db(), &job_id).await?;

    if !cancelled {
        return Err(AppError::Conflict(
            "Job not found or cannot be cancelled (only pending/claimed jobs can be cancelled)"
                .into(),
        ));
    }

    let job = JobRepo::get_by_id(state.db(), &job_id).await?;
    Ok(Json(job))
}

/// PUT /api/admin/jobs/{id}/retry
async fn retry_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    let retried = JobRepo::retry(state.db(), &job_id).await?;

    if !retried {
        return Err(AppError::Conflict(
            "Job not found or cannot be retried (only failed jobs can be retried)".into(),
        ));
    }

    let job = JobRepo::get_by_id(state.db(), &job_id).await?;
    Ok(Json(job))
}

/// PUT /api/admin/jobs/{id}/release
async fn release_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    let released = JobRepo::release(state.db(), &job_id).await?;

    if !released {
        return Err(AppError::Conflict(
            "Job not found or cannot be released (only claimed/running jobs can be released)"
                .into(),
        ));
    }

    let job = JobRepo::get_by_id(state.db(), &job_id).await?;
    Ok(Json(job))
}
