use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{
        CompleteWorldUploadRequest, CreateWorldRequest, CreateWorldUploadResponse,
        UpdateWorldRequest, World, WorldWithScenes,
    },
    repo::{PendingUploadRepo, SceneRepo, WorldRepo},
    state::AppState,
};

/// Presigned URL expiry time (5 minutes)
const PRESIGN_EXPIRY_SECS: u64 = 300;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_worlds).post(create_world_upload))
        .route(
            "/{id}",
            get(get_world).put(update_world).delete(delete_world),
        )
        .route("/{slug}/complete", post(complete_world_upload))
}

/// GET /api/worlds - List all worlds (admin)
async fn list_worlds(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<World>>> {
    let worlds = WorldRepo::list(state.db()).await?;
    Ok(Json(worlds))
}

/// POST /api/worlds - Initiate world upload (admin)
async fn create_world_upload(
    _admin: AdminUser,
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

/// POST /api/worlds/{slug}/complete - Complete world upload (admin)
async fn complete_world_upload(
    _admin: AdminUser,
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

/// GET /api/worlds/{id} - Get world with scenes (admin)
async fn get_world(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<WorldWithScenes>> {
    let world = WorldRepo::get_by_id(state.db(), &id).await?;
    let scenes = SceneRepo::list_by_world(state.db(), &id).await?;
    Ok(Json(WorldWithScenes { world, scenes }))
}

/// PUT /api/worlds/{id} - Update world metadata (admin)
async fn update_world(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateWorldRequest>,
) -> AppResult<Json<WorldWithScenes>> {
    let world = WorldRepo::update(state.db(), &id, &request).await?;
    let scenes = SceneRepo::list_by_world(state.db(), &id).await?;
    Ok(Json(WorldWithScenes { world, scenes }))
}

/// DELETE /api/worlds/{id} - Delete a world (admin)
async fn delete_world(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let world = WorldRepo::get_by_id(state.db(), &id).await?;
    let deleted = WorldRepo::delete(state.db(), &id).await?;
    if !deleted {
        return Err(AppError::NotFound("World not found".into()));
    }

    if let Some(s3) = state.s3() {
        let r2_config = &state.config().r2;
        let bucket = r2_config.bucket.as_deref().unwrap_or("glint");
        let r2_key = format!("worlds/{}.zip", world.slug);
        if let Err(e) = s3.delete_object().bucket(bucket).key(&r2_key).send().await {
            warn!(key = %r2_key, error = %e, "Failed to delete world file from R2");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
