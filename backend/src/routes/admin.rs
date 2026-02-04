use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{
        CompleteWorldUploadRequest, CreateJobRequest, CreateSceneRequest, CreateShaderRequest,
        CreateShaderVersionRequest, CreateWorldUploadRequest, CreateWorldUploadResponse, Job,
        PendingUpload, Scene, Shader, ShaderVersion, UpdateSceneRequest, World,
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

    let result = sqlx::query!(
        r#"
        INSERT INTO shaders (id, name, slug, description, modrinth_id, curseforge_id, website_url, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
        "#,
        id,
        request.name,
        request.slug,
        request.description,
        request.modrinth_id,
        request.curseforge_id,
        request.website_url
    )
    .execute(state.db())
    .await;

    if let Err(sqlx::Error::Database(db_err)) = &result
        && let Some(code) = db_err.code()
        && code == "23505"
    {
        return Err(crate::error::AppError::Conflict(format!(
            "Shader with slug '{}' already exists",
            request.slug
        )));
    }
    result?;

    let shader = sqlx::query_as!(Shader, "SELECT * FROM shaders WHERE id = $1", id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(shader)))
}

/// POST /api/admin/shaders/{id}/versions
async fn create_shader_version(
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
    Json(request): Json<CreateShaderVersionRequest>,
) -> AppResult<(StatusCode, Json<ShaderVersion>)> {
    // Verify shader exists
    let exists = sqlx::query_scalar!("SELECT 1 as one FROM shaders WHERE id = $1", shader_id)
        .fetch_optional(state.db())
        .await?;

    if exists.is_none() {
        return Err(crate::error::AppError::NotFound("Shader not found".into()));
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query!(
        r#"
        INSERT INTO shader_versions (id, shader_id, version, modrinth_version_id, download_url, file_hash, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        "#,
        id,
        shader_id,
        request.version,
        request.modrinth_version_id,
        request.download_url,
        request.file_hash
    )
    .execute(state.db())
    .await?;

    let version = sqlx::query_as!(
        ShaderVersion,
        "SELECT * FROM shader_versions WHERE id = $1",
        id
    )
    .fetch_one(state.db())
    .await?;

    Ok((StatusCode::CREATED, Json(version)))
}

/// GET /api/admin/worlds
async fn list_worlds(State(state): State<AppState>) -> AppResult<Json<Vec<World>>> {
    let worlds = sqlx::query_as!(World, "SELECT * FROM worlds ORDER BY created_at DESC")
        .fetch_all(state.db())
        .await?;

    Ok(Json(worlds))
}

/// POST /api/admin/worlds
/// Initiates a world upload by returning a presigned URL
async fn create_world_upload(
    State(state): State<AppState>,
    Json(request): Json<CreateWorldUploadRequest>,
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
    let exists = sqlx::query_scalar!("SELECT 1 as one FROM worlds WHERE slug = $1", request.slug)
        .fetch_optional(state.db())
        .await?;

    if exists.is_some() {
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
    sqlx::query!(
        r#"
        INSERT INTO pending_uploads (upload_id, slug, name, description, minecraft_version, file_hash, size_bytes, upload_key, expires_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
        "#,
        upload_id,
        request.slug,
        request.name,
        request.description,
        request.minecraft_version,
        request.file_hash,
        request.file_size_bytes,
        upload_key,
        expires_at
    )
    .execute(state.db())
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

    // Fetch pending upload record
    let pending = sqlx::query_as!(
        PendingUpload,
        "SELECT * FROM pending_uploads WHERE upload_id = $1",
        request.upload_id
    )
    .fetch_optional(state.db())
    .await?;

    let pending = pending.ok_or_else(|| {
        AppError::NotFound(format!("Upload ID '{}' not found", request.upload_id))
    })?;

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
        sqlx::query!(
            "DELETE FROM pending_uploads WHERE upload_id = $1",
            request.upload_id
        )
        .execute(state.db())
        .await?;

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

    // Create world record (atomic - first to complete wins)
    let world_id = Uuid::new_v4().to_string();

    let result = sqlx::query!(
        r#"
        INSERT INTO worlds (id, name, slug, description, minecraft_version, file_url, file_hash, size_bytes, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
        "#,
        world_id,
        pending.name,
        pending.slug,
        pending.description,
        pending.minecraft_version,
        file_url,
        pending.file_hash,
        pending.size_bytes
    )
    .execute(state.db())
    .await;

    // Handle conflict (another upload completed first)
    if let Err(sqlx::Error::Database(db_err)) = &result
        && let Some(code) = db_err.code()
        && code == "23505"
    {
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
        sqlx::query!(
            "DELETE FROM pending_uploads WHERE upload_id = $1",
            request.upload_id
        )
        .execute(state.db())
        .await?;

        return Err(AppError::Conflict(format!(
            "World with slug '{}' was created by another upload",
            slug
        )));
    }
    result?;

    // Delete pending upload record
    sqlx::query!(
        "DELETE FROM pending_uploads WHERE upload_id = $1",
        request.upload_id
    )
    .execute(state.db())
    .await?;

    // Fetch created world
    let world = sqlx::query_as!(World, "SELECT * FROM worlds WHERE id = $1", world_id)
        .fetch_one(state.db())
        .await?;

    info!(world_id = %world.id, slug = %world.slug, "World created");
    Ok((StatusCode::CREATED, Json(world)))
}

/// POST /api/admin/scenes
async fn create_scene(
    State(state): State<AppState>,
    Json(request): Json<CreateSceneRequest>,
) -> AppResult<(StatusCode, Json<Scene>)> {
    // Verify world exists
    let exists = sqlx::query_scalar!(
        "SELECT 1 as one FROM worlds WHERE id = $1",
        request.world_id
    )
    .fetch_optional(state.db())
    .await?;

    if exists.is_none() {
        return Err(AppError::NotFound("World not found".into()));
    }

    // Check world-scoped slug uniqueness (only active scenes)
    let slug_exists = sqlx::query_scalar!(
        "SELECT 1 as one FROM scenes WHERE world_id = $1 AND slug = $2 AND active = TRUE",
        request.world_id,
        request.slug
    )
    .fetch_optional(state.db())
    .await?;

    if slug_exists.is_some() {
        return Err(AppError::Conflict(format!(
            "Scene with slug '{}' already exists in this world",
            request.slug
        )));
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query!(
        r#"
        INSERT INTO scenes (
            id, name, slug, world_id, x, y, z, pitch, yaw,
            dimension, time_of_day_ticks, weather, weather_intensity, moon_phase, biome,
            active, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, TRUE, now())
        "#,
        id,
        request.name,
        request.slug,
        request.world_id,
        request.position.x,
        request.position.y,
        request.position.z,
        request.camera.pitch,
        request.camera.yaw,
        request.dimension,
        request.time_of_day,
        request.weather,
        request.weather_intensity,
        request.moon_phase,
        request.biome
    )
    .execute(state.db())
    .await?;

    let scene = sqlx::query_as!(Scene, "SELECT * FROM scenes WHERE id = $1", id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(scene)))
}

/// PUT /api/admin/scenes/{slug}
async fn update_scene(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateSceneRequest>,
) -> AppResult<Json<Scene>> {
    // Find scene (world-scoped, active only)
    let scene = sqlx::query_as!(
        Scene,
        "SELECT * FROM scenes WHERE slug = $1 AND world_id = $2 AND active = TRUE",
        slug,
        request.world_id
    )
    .fetch_optional(state.db())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Scene '{}' not found in this world", slug)))?;

    // Update position/camera/environment fields only (preserve name, description)
    sqlx::query!(
        r#"
        UPDATE scenes
        SET x = $1, y = $2, z = $3, pitch = $4, yaw = $5,
            dimension = $6, time_of_day_ticks = $7, weather = $8,
            weather_intensity = $9, moon_phase = $10, biome = $11
        WHERE id = $12
        "#,
        request.position.x,
        request.position.y,
        request.position.z,
        request.camera.pitch,
        request.camera.yaw,
        request.dimension,
        request.time_of_day,
        request.weather,
        request.weather_intensity,
        request.moon_phase,
        request.biome,
        scene.id
    )
    .execute(state.db())
    .await?;

    // Mark captures as outdated
    sqlx::query!(
        "UPDATE captures SET outdated = TRUE WHERE scene_id = $1 AND status = 'completed'",
        scene.id
    )
    .execute(state.db())
    .await?;

    // Fetch updated scene
    let updated = sqlx::query_as!(Scene, "SELECT * FROM scenes WHERE id = $1", scene.id)
        .fetch_one(state.db())
        .await?;

    Ok(Json(updated))
}

/// DELETE /api/admin/scenes/{slug}
async fn disable_scene(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<WorldIdParam>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!(
        "UPDATE scenes SET active = FALSE WHERE slug = $1 AND world_id = $2 AND active = TRUE",
        slug,
        params.world_id
    )
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Scene '{}' not found", slug)));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/admin/jobs
async fn list_jobs(State(state): State<AppState>) -> AppResult<Json<Vec<JobWithDetails>>> {
    let jobs = sqlx::query_as!(
        JobWithDetails,
        r#"
        SELECT
            j.id, j.shader_version_id, j.scene_ids, j.profiles, j.priority,
            j.status, j.attempts, j.max_attempts, j.agent_id, j.claimed_at,
            j.last_heartbeat, j.started_at, j.completed_at, j.error_message,
            j.created_at,
            s.name as shader_name,
            s.slug as shader_slug,
            sv.version as shader_version,
            COALESCE(json_array_length(j.scene_ids::json), 0)::int4 as "scene_count!"
        FROM jobs j
        JOIN shader_versions sv ON sv.id = j.shader_version_id
        JOIN shaders s ON s.id = sv.shader_id
        ORDER BY j.created_at DESC
        "#,
    )
    .fetch_all(state.db())
    .await?;

    Ok(Json(jobs))
}

/// POST /api/admin/jobs
async fn create_job(
    State(state): State<AppState>,
    Json(request): Json<CreateJobRequest>,
) -> AppResult<(StatusCode, Json<Job>)> {
    // Verify shader version exists
    let exists = sqlx::query_scalar!(
        "SELECT 1 as one FROM shader_versions WHERE id = $1",
        request.shader_version_id
    )
    .fetch_optional(state.db())
    .await?;

    if exists.is_none() {
        return Err(crate::error::AppError::NotFound(
            "Shader version not found".into(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let scene_ids_json =
        serde_json::to_string(&request.scene_ids).unwrap_or_else(|_| "[]".to_string());
    let profiles_json = request
        .profiles
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_else(|_| "[]".to_string()));
    let priority = request.priority.unwrap_or(0);

    sqlx::query!(
        r#"
        INSERT INTO jobs (id, shader_version_id, scene_ids, profiles, priority, status, attempts, max_attempts, created_at)
        VALUES ($1, $2, $3, $4, $5, 'pending', 0, 3, now())
        "#,
        id,
        request.shader_version_id,
        scene_ids_json,
        profiles_json,
        priority
    )
    .execute(state.db())
    .await?;

    let job = sqlx::query_as!(Job, "SELECT * FROM jobs WHERE id = $1", id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(job)))
}

#[derive(Deserialize)]
struct WorldIdParam {
    world_id: String,
}

#[derive(sqlx::FromRow, Serialize)]
struct JobWithDetails {
    id: String,
    shader_version_id: String,
    scene_ids: Option<String>,
    profiles: Option<String>,
    priority: i32,
    status: String,
    attempts: i32,
    max_attempts: i32,
    agent_id: Option<String>,
    claimed_at: Option<DateTime<Utc>>,
    last_heartbeat: Option<DateTime<Utc>>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    shader_name: String,
    shader_slug: String,
    shader_version: String,
    scene_count: i32,
}

/// DELETE /api/admin/jobs/{id}
async fn delete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
        .execute(state.db())
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::NotFound("Job not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/admin/jobs/{id}/cancel
async fn cancel_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    // Only cancel pending or claimed jobs
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'failed',
            error_message = 'Cancelled by admin',
            completed_at = now()
        WHERE id = $1 AND status IN ('pending', 'claimed')
        "#,
        job_id
    )
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::Conflict(
            "Job not found or cannot be cancelled (only pending/claimed jobs can be cancelled)"
                .into(),
        ));
    }

    let job = sqlx::query_as!(Job, "SELECT * FROM jobs WHERE id = $1", job_id)
        .fetch_one(state.db())
        .await?;

    Ok(Json(job))
}

/// PUT /api/admin/jobs/{id}/retry
async fn retry_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    // Only retry failed jobs
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'pending',
            attempts = 0,
            error_message = NULL,
            agent_id = NULL,
            claimed_at = NULL,
            last_heartbeat = NULL,
            started_at = NULL,
            completed_at = NULL
        WHERE id = $1 AND status = 'failed'
        "#,
        job_id
    )
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::Conflict(
            "Job not found or cannot be retried (only failed jobs can be retried)".into(),
        ));
    }

    let job = sqlx::query_as!(Job, "SELECT * FROM jobs WHERE id = $1", job_id)
        .fetch_one(state.db())
        .await?;

    Ok(Json(job))
}

/// PUT /api/admin/jobs/{id}/release
async fn release_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    // Release claimed or running jobs
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'pending',
            agent_id = NULL,
            claimed_at = NULL,
            last_heartbeat = NULL
        WHERE id = $1 AND status IN ('claimed', 'running')
        "#,
        job_id
    )
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::Conflict(
            "Job not found or cannot be released (only claimed/running jobs can be released)"
                .into(),
        ));
    }

    let job = sqlx::query_as!(Job, "SELECT * FROM jobs WHERE id = $1", job_id)
        .fetch_one(state.db())
        .await?;

    Ok(Json(job))
}
