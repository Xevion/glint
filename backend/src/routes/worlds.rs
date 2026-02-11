use crate::{
    auth::AdminUser,
    config::R2Config,
    error::{AppError, AppResult, OptionNotFoundExt},
    models::{
        CompleteUploadRequest, CreateWorldRequest, CreateWorldVersionUploadRequest, PendingUpload,
        UpdateWorldRequest, UploadResponse, World, WorldListItem, WorldPreviewCapture,
        WorldVersion, WorldWithDetails,
    },
    repo::{PendingUploadRepo, SceneRepo, WorldRepo, WorldVersionRepo},
    state::AppState,
};
use aws_sdk_s3::Client as S3Client;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use tracing::{debug, error, info, instrument, warn};

/// Presigned URL expiry time (5 minutes)
const PRESIGN_EXPIRY_SECS: u64 = 300;

/// Maximum upload size (512 MiB)
const MAX_UPLOAD_SIZE: i64 = 512 * 1024 * 1024;

/// Validate common upload fields (hash format, file size).
fn validate_upload_request(file_hash: &str, file_size_bytes: i64) -> AppResult<()> {
    if !file_hash.starts_with("sha256:") {
        return Err(AppError::BadRequest(
            "file_hash must have algorithm prefix (e.g., 'sha256:abc123...')".into(),
        ));
    }
    if file_size_bytes <= 0 {
        return Err(AppError::BadRequest(
            "file_size_bytes must be greater than zero".into(),
        ));
    }
    if file_size_bytes > MAX_UPLOAD_SIZE {
        return Err(AppError::BadRequest(format!(
            "File too large: {} bytes (max {} MiB)",
            file_size_bytes,
            MAX_UPLOAD_SIZE / 1024 / 1024
        )));
    }
    Ok(())
}

/// Result of generating a presigned upload URL for S3 staging.
struct PresignedUpload {
    upload_id: String,
    upload_key: String,
    presigned_url: String,
    expires_at: chrono::DateTime<Utc>,
}

/// Generate a presigned PUT URL for uploading a file to the S3 staging path.
async fn generate_presigned_upload(
    s3: &S3Client,
    bucket: &str,
    file_hash: &str,
) -> AppResult<PresignedUpload> {
    let upload_id = crate::id::generate_id();
    let upload_key = format!("_uploads/{}.zip", upload_id);
    let expires_at = Utc::now() + chrono::Duration::seconds(PRESIGN_EXPIRY_SECS as i64);

    let hash_value = file_hash.strip_prefix("sha256:").unwrap_or(file_hash);

    let presigned = s3
        .put_object()
        .bucket(bucket)
        .key(&upload_key)
        .content_type("application/zip")
        .metadata("sha256", hash_value)
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

    Ok(PresignedUpload {
        upload_id,
        upload_key,
        presigned_url: presigned.uri().to_string(),
        expires_at,
    })
}

/// Require S3 to be configured, returning a service-unavailable error if not.
fn require_s3(state: &AppState) -> AppResult<&S3Client> {
    state
        .s3()
        .ok_or_else(|| AppError::ServiceUnavailable("R2/S3 storage not configured".into()))
}

/// Result of successfully verifying and moving a staged upload in S3.
struct FinalizedUpload {
    version_id: String,
    file_url: String,
}

/// Verify a staged upload exists in S3 with the correct hash, then move it
/// from the staging path to the final versioned path and return the public URL.
async fn finalize_staged_upload(
    s3: &S3Client,
    r2_config: &R2Config,
    pending: &PendingUpload,
    world_slug: &str,
) -> AppResult<FinalizedUpload> {
    let bucket = r2_config.bucket.as_deref().unwrap_or("glint");

    // Verify file exists in R2
    let head = s3
        .head_object()
        .bucket(bucket)
        .key(&pending.upload_key)
        .send()
        .await
        .map_err(|e| {
            warn!(upload_key = %pending.upload_key, error = %e, "Upload file not found in R2");
            AppError::NotFound(
                "Uploaded file not found in storage. Please retry the upload.".into(),
            )
        })?;

    // Verify hash from metadata
    let stored_hash = head.metadata().and_then(|m| m.get("sha256").cloned());
    let expected_hash = pending
        .file_hash
        .strip_prefix("sha256:")
        .unwrap_or(&pending.file_hash);

    if stored_hash.as_deref() != Some(expected_hash) {
        warn!(
            expected = %expected_hash,
            actual = ?stored_hash,
            "Upload hash mismatch"
        );
        return Err(AppError::BadRequest(
            "File hash mismatch. The uploaded file may be corrupted.".into(),
        ));
    }

    // Move file to versioned location
    let version_id = crate::id::generate_id();
    let final_key = format!("worlds/{}/{}.zip", world_slug, version_id);

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

    // Delete staging file (best-effort)
    if let Err(e) = s3
        .delete_object()
        .bucket(bucket)
        .key(&pending.upload_key)
        .send()
        .await
    {
        warn!(key = %pending.upload_key, error = %e, "Failed to delete temp upload file");
    }

    // Generate public URL
    let file_url = if let Some(ref public_url_prefix) = r2_config.public_url {
        format!("{}/{}", public_url_prefix, final_key)
    } else {
        format!("https://{}.r2.cloudflarestorage.com/{}", bucket, final_key)
    };

    Ok(FinalizedUpload {
        version_id,
        file_url,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_worlds).post(create_world_upload))
        .route(
            "/{id}",
            get(get_world).put(update_world).delete(delete_world),
        )
        .route(
            "/{id}/versions",
            get(list_world_versions).post(create_world_version),
        )
        .route(
            "/{id}/versions/complete",
            post(complete_world_version_upload),
        )
        .route("/{slug}/complete", post(complete_world_upload))
}

/// GET /api/worlds - List all worlds with latest version (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn list_worlds(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<WorldListItem>>> {
    let db = state.db();

    let (worlds, latest_versions, aggregates, previews) = tokio::try_join!(
        WorldRepo::list(db),
        WorldVersionRepo::batch_latest(db),
        WorldRepo::aggregate_counts(db),
        WorldRepo::preview_captures(db),
    )?;

    let items =
        worlds
            .into_iter()
            .map(|world| {
                let latest_version = latest_versions
                    .iter()
                    .find(|v| v.world_id == world.id)
                    .cloned();
                let agg = aggregates.iter().find(|a| a.world_id == world.id.0);
                let preview = previews.iter().find(|p| p.world_id == world.id.0).map(|p| {
                    WorldPreviewCapture {
                        image_url: p.image_url.clone(),
                        thumbhash: p.thumbhash.clone(),
                    }
                });
                WorldListItem {
                    world,
                    latest_version,
                    scene_count: agg.map_or(0, |a| a.scene_count),
                    version_count: agg.map_or(0, |a| a.version_count),
                    capture_count: agg.map_or(0, |a| a.capture_count),
                    preview,
                }
            })
            .collect();

    Ok(Json(items))
}

/// POST /api/worlds - Initiate world upload (admin)
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn create_world_upload(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<crate::models::CreateWorldUploadRequest>,
) -> AppResult<(StatusCode, Json<UploadResponse>)> {
    debug!(slug = %request.slug, "Preparing world upload");

    validate_upload_request(&request.file_hash, request.file_size_bytes)?;

    // Check if world with this slug already exists
    if WorldRepo::exists_by_slug(state.db(), &request.slug).await? {
        return Err(AppError::Conflict(format!(
            "World with slug '{}' already exists",
            request.slug
        )));
    }

    let s3 = require_s3(&state)?;
    let bucket = state.config().r2.bucket.as_deref().unwrap_or("glint");

    let upload = generate_presigned_upload(s3, bucket, &request.file_hash).await?;

    // Store pending upload record
    PendingUploadRepo::create(
        state.db(),
        &upload.upload_id,
        &request.slug,
        &request.name,
        request.description.as_deref(),
        &request.minecraft_version,
        &request.file_hash,
        request.file_size_bytes,
        &upload.upload_key,
        upload.expires_at,
    )
    .await?;

    debug!(
        upload_id = %upload.upload_id,
        slug = %request.slug,
        expires_at = %upload.expires_at,
        "Pending upload created"
    );

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            upload_id: upload.upload_id,
            presigned_url: upload.presigned_url,
            expires_at: upload.expires_at,
        }),
    ))
}

/// POST /api/worlds/{slug}/complete - Complete world upload (admin)
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn complete_world_upload(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<CompleteUploadRequest>,
) -> AppResult<(StatusCode, Json<World>)> {
    debug!(slug = %slug, upload_id = %request.upload_id, "Completing world upload");

    let pending = PendingUploadRepo::find_by_id(state.db(), &request.upload_id)
        .await?
        .or_not_found("Upload", &request.upload_id)?;

    // This endpoint is for world creation uploads (not version uploads)
    let pending_slug = pending.slug.as_deref().ok_or_else(|| {
        AppError::BadRequest("Upload ID is for a version upload, not world creation".into())
    })?;
    let pending_name = pending
        .name
        .clone()
        .ok_or_else(|| AppError::BadRequest("Pending upload missing name".into()))?;
    let pending_mc_version = pending
        .minecraft_version
        .clone()
        .ok_or_else(|| AppError::BadRequest("Pending upload missing minecraft_version".into()))?;

    // Verify slug matches
    if pending_slug != slug {
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

    let s3 = require_s3(&state)?;
    let r2_config = &state.config().r2;

    // Verify and move staged file in S3
    let finalized = finalize_staged_upload(s3, r2_config, &pending, pending_slug).await?;

    // Create world + version + cleanup pending record in a single transaction
    let mut tx = state.begin_tx().await?;

    let world_id = crate::id::generate_id();
    let world_result = WorldRepo::create(
        &mut *tx,
        &world_id,
        &CreateWorldRequest {
            name: &pending_name,
            slug: pending_slug,
            description: pending.description.as_deref(),
            minecraft_version: &pending_mc_version,
        },
    )
    .await;

    // Handle conflict (another upload completed first)
    if let Err(AppError::Conflict(_)) = &world_result {
        tx.rollback().await.ok();
        PendingUploadRepo::delete(state.db(), &request.upload_id).await?;
        return Err(AppError::Conflict(format!(
            "World with slug '{}' was created by another upload",
            slug
        )));
    }

    let world = world_result?;

    WorldVersionRepo::create(
        &mut *tx,
        &finalized.version_id,
        world.id.as_ref(),
        &finalized.file_url,
        &pending.file_hash,
        pending.size_bytes,
    )
    .await?;

    PendingUploadRepo::delete(&mut *tx, &request.upload_id).await?;
    tx.commit().await?;

    info!(world_id = %world.id, slug = %world.slug, "World created with initial version");
    Ok((StatusCode::CREATED, Json(world)))
}

/// GET /api/worlds/{id} - Get world with scenes and latest version (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn get_world(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<WorldWithDetails>> {
    let (world, scenes, latest_version) = tokio::try_join!(
        async {
            WorldRepo::find_by_id(state.db(), &id)
                .await?
                .or_not_found("World", &id)
        },
        SceneRepo::list_by_world(state.db(), &id),
        WorldVersionRepo::get_latest_for_world(state.db(), &id),
    )?;
    Ok(Json(WorldWithDetails {
        world,
        scenes,
        latest_version,
    }))
}

/// PUT /api/worlds/{id} - Update world metadata (admin)
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn update_world(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateWorldRequest>,
) -> AppResult<Json<WorldWithDetails>> {
    let world = WorldRepo::update(state.db(), &id, &request).await?;
    let (scenes, latest_version) = tokio::try_join!(
        SceneRepo::list_by_world(state.db(), &id),
        WorldVersionRepo::get_latest_for_world(state.db(), &id),
    )?;
    Ok(Json(WorldWithDetails {
        world,
        scenes,
        latest_version,
    }))
}

/// DELETE /api/worlds/{id} - Delete a world (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn delete_world(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    // Verify world exists (404 if not)
    WorldRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("World", &id)?;

    // Fetch versions before deletion (CASCADE will remove them from DB)
    let versions = WorldVersionRepo::list_by_world(state.db(), &id).await?;

    let deleted = WorldRepo::delete(state.db(), &id).await?;
    if !deleted {
        return Err(AppError::NotFound("World not found".into()));
    }

    // Clean up R2 files for all versions
    if let Some(s3) = state.s3() {
        let r2_config = &state.config().r2;
        let bucket = r2_config.bucket.as_deref().unwrap_or("glint");

        let public_prefix = r2_config
            .public_url
            .as_deref()
            .map(|p| format!("{}/", p))
            .unwrap_or_else(|| format!("https://{}.r2.cloudflarestorage.com/", bucket));

        for version in &versions {
            if let Some(ref url) = version.file_url {
                // Extract R2 key by stripping the public URL prefix
                let key = url.strip_prefix(&public_prefix).unwrap_or_else(|| {
                    // Fallback: legacy canonical path
                    warn!(url = %url, "Could not extract R2 key from file_url, skipping");
                    ""
                });

                if !key.is_empty()
                    && let Err(e) = s3.delete_object().bucket(bucket).key(key).send().await
                {
                    warn!(key = %key, error = %e, "Failed to delete version file from R2");
                }
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/worlds/{id}/versions - List all versions for a world (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn list_world_versions(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<WorldVersion>>> {
    // Verify world exists (404 if not)
    WorldRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("World", &id)?;
    let versions = WorldVersionRepo::list_by_world(state.db(), &id).await?;
    Ok(Json(versions))
}

/// POST /api/worlds/{id}/versions - Initiate world version upload (admin)
///
/// Phase 1 of two-phase upload: creates a pending upload record and returns
/// a presigned URL for uploading to a staging path. The client must call
/// `POST /api/worlds/{id}/versions/complete` after uploading.
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn create_world_version(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<CreateWorldVersionUploadRequest>,
) -> AppResult<(StatusCode, Json<UploadResponse>)> {
    let world = WorldRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("World", &id)?;

    validate_upload_request(&request.file_hash, request.file_size_bytes)?;

    let s3 = require_s3(&state)?;
    let bucket = state.config().r2.bucket.as_deref().unwrap_or("glint");

    let upload = generate_presigned_upload(s3, bucket, &request.file_hash).await?;

    // Store pending upload record
    PendingUploadRepo::create_for_version(
        state.db(),
        &upload.upload_id,
        world.id.as_ref(),
        &request.file_hash,
        request.file_size_bytes,
        &upload.upload_key,
        upload.expires_at,
    )
    .await?;

    debug!(
        upload_id = %upload.upload_id,
        world_id = %world.id,
        expires_at = %upload.expires_at,
        "Pending version upload created"
    );

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            upload_id: upload.upload_id,
            presigned_url: upload.presigned_url,
            expires_at: upload.expires_at,
        }),
    ))
}

/// POST /api/worlds/{id}/versions/complete - Complete world version upload (admin)
///
/// Phase 2 of two-phase upload: verifies the file was uploaded to R2,
/// checks the hash, moves it to a versioned path, and creates the
/// WorldVersion database record.
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn complete_world_version_upload(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(world_id): Path<String>,
    Json(request): Json<CompleteUploadRequest>,
) -> AppResult<(StatusCode, Json<WorldVersion>)> {
    debug!(world_id = %world_id, upload_id = %request.upload_id, "Completing world version upload");

    let pending = PendingUploadRepo::find_by_id(state.db(), &request.upload_id)
        .await?
        .or_not_found("Upload", &request.upload_id)?;

    // Verify this is a version upload and world_id matches
    let pending_world_id = pending.world_id.as_deref().ok_or_else(|| {
        AppError::BadRequest("Upload ID is for world creation, not a version upload".into())
    })?;
    if pending_world_id != world_id {
        return Err(AppError::BadRequest(format!(
            "Upload ID '{}' does not match world '{}'",
            request.upload_id, world_id
        )));
    }

    // Check if expired
    if pending.expires_at < Utc::now() {
        PendingUploadRepo::delete(state.db(), &request.upload_id).await?;
        return Err(AppError::Gone(
            "Upload has expired. Please start a new upload.".into(),
        ));
    }

    // Verify world still exists
    let world = WorldRepo::find_by_id(state.db(), &world_id)
        .await?
        .or_not_found("World", &world_id)?;

    let s3 = require_s3(&state)?;
    let r2_config = &state.config().r2;

    // Verify and move staged file in S3
    let finalized = finalize_staged_upload(s3, r2_config, &pending, &world.slug).await?;

    // Create version + cleanup pending record in a single transaction
    let mut tx = state.begin_tx().await?;

    let version = WorldVersionRepo::create(
        &mut *tx,
        &finalized.version_id,
        world.id.as_ref(),
        &finalized.file_url,
        &pending.file_hash,
        pending.size_bytes,
    )
    .await?;

    PendingUploadRepo::delete(&mut *tx, &request.upload_id).await?;
    tx.commit().await?;

    info!(
        world_id = %world.id,
        version_id = %version.id,
        "World version created and upload finalized"
    );
    Ok((StatusCode::CREATED, Json(version)))
}
