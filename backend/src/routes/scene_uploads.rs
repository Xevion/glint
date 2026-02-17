use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use tracing::{info, instrument};
use validator::Validate;

use anyhow::Context;

use crate::auth::AgentUser;
use crate::error::{AppError, AppResult, OptionNotFoundExt};
use crate::id::{PendingSceneUploadId, ScenePresetId};
use crate::models::{
    CompleteUploadRequest, CompleteUploadResponse, InitiateNewSceneUploadRequest,
    InitiateVersionUploadRequest, UploadInitiatedResponse,
};
use crate::repo::{PendingSceneUploadRepo, ScenePresetRepo, SceneRepo, SceneVersionRepo};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(initiate_new_scene_upload))
        .route("/{slug}/versions", post(initiate_version_upload))
        .route("/uploads/{upload_id}/complete", post(complete_scene_upload))
}

/// Generate a presigned PUT URL for an R2 object.
async fn generate_presigned_put_url(
    s3: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    content_type: &str,
    expiry_secs: u64,
) -> AppResult<String> {
    let presigned = s3
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::builder()
                .expires_in(std::time::Duration::from_secs(expiry_secs))
                .build()
                .expect("valid presigning config"),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to generate presigned URL: {e}"))?;
    Ok(presigned.uri().to_string())
}

/// Helper to get R2 client and bucket, returning appropriate errors.
fn require_r2(state: &AppState) -> AppResult<(&aws_sdk_s3::Client, &str)> {
    let s3 = state
        .s3()
        .ok_or_else(|| AppError::ServiceUnavailable("R2/S3 storage not configured".into()))?;
    let bucket = state
        .config()
        .r2
        .bucket
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("R2 bucket not configured".into()))?;
    Ok((s3, bucket))
}

/// POST /api/scenes/upload — Initiate upload for a new scene (agent).
///
/// Creates a pending_scene_upload record with scene metadata and returns
/// a presigned R2 URL for the scene package zip.
#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn initiate_new_scene_upload(
    _user: AgentUser,
    State(state): State<AppState>,
    Json(request): Json<InitiateNewSceneUploadRequest>,
) -> AppResult<(StatusCode, Json<UploadInitiatedResponse>)> {
    request.validate()?;

    // Check slug uniqueness (active scenes)
    if SceneRepo::exists_by_slug(state.db(), &request.slug).await? {
        return Err(AppError::Conflict(format!(
            "Scene with slug '{}' already exists",
            request.slug
        )));
    }

    let (s3, bucket) = require_r2(&state)?;

    let upload_id = PendingSceneUploadId::generate();
    let r2_key = format!("scene-packages/{}/{}.zip", request.slug, upload_id);

    let upload_url =
        generate_presigned_put_url(s3, bucket, &r2_key, "application/zip", 3600).await?;

    PendingSceneUploadRepo::create_for_new_scene(
        state.db(),
        &upload_id,
        &request.name,
        &request.slug,
        &request.dimension,
        request.description.as_deref(),
        &request.minecraft_version,
        &request.file_hash,
        request.size_bytes,
        &r2_key,
    )
    .await?;

    info!(
        upload_id = %upload_id,
        slug = %request.slug,
        "Initiated scene package upload for new scene"
    );

    Ok((
        StatusCode::CREATED,
        Json(UploadInitiatedResponse {
            upload_id,
            upload_url,
        }),
    ))
}

/// POST /api/scenes/{slug}/versions — Initiate upload for an existing scene (agent).
///
/// Creates a pending_scene_upload record for a new version and returns
/// a presigned R2 URL for the scene package zip.
#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn initiate_version_upload(
    _user: AgentUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<InitiateVersionUploadRequest>,
) -> AppResult<(StatusCode, Json<UploadInitiatedResponse>)> {
    request.validate()?;

    let scene = SceneRepo::find_active_by_slug(state.db(), &slug)
        .await?
        .into_iter()
        .next()
        .or_not_found("Scene", &slug)?;

    let (s3, bucket) = require_r2(&state)?;

    let upload_id = PendingSceneUploadId::generate();
    let r2_key = format!("scene-packages/{}/{}.zip", slug, upload_id);

    let upload_url =
        generate_presigned_put_url(s3, bucket, &r2_key, "application/zip", 3600).await?;

    PendingSceneUploadRepo::create_for_existing_scene(
        state.db(),
        &upload_id,
        scene.id.as_ref(),
        &request.minecraft_version,
        &request.file_hash,
        request.size_bytes,
        &r2_key,
    )
    .await?;

    info!(
        upload_id = %upload_id,
        scene_id = %scene.id,
        "Initiated scene package upload for existing scene"
    );

    Ok((
        StatusCode::CREATED,
        Json(UploadInitiatedResponse {
            upload_id,
            upload_url,
        }),
    ))
}

/// POST /api/scenes/uploads/{upload_id}/complete — Confirm upload and create scene version.
///
/// For new scenes: creates scene + scene_version + default preset atomically.
/// For existing scenes: creates a new scene_version.
#[instrument(skip(state, _user, request), fields(user_id = _user.user.id))]
async fn complete_scene_upload(
    _user: AgentUser,
    State(state): State<AppState>,
    Path(upload_id): Path<String>,
    Json(request): Json<CompleteUploadRequest>,
) -> AppResult<Json<CompleteUploadResponse>> {
    request.validate()?;

    let pending = PendingSceneUploadRepo::find_by_id(state.db(), &upload_id)
        .await?
        .or_not_found("Pending upload", &upload_id)?;

    // Verify file hash matches
    if request.file_hash != pending.file_hash {
        return Err(AppError::BadRequest(format!(
            "File hash mismatch: expected '{}', got '{}'",
            pending.file_hash, request.file_hash
        )));
    }

    let r2_config = &state.config().r2;
    let package_url = r2_config.public_url_for_key(&pending.r2_key);

    let is_new_scene = pending.scene_id.is_none();

    // Use a transaction for atomic scene + version + preset creation
    let pool = state.db();
    let mut tx = pool.begin().await.context("failed to begin transaction")?;

    // Create scene if this is a new scene upload
    let scene_id: crate::id::SceneId = if let Some(existing_id) = pending.scene_id.clone() {
        crate::id::SceneId(existing_id)
    } else {
        let id = crate::id::generate_id();
        let scene_name = pending.scene_name.as_deref().ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("missing scene_name in pending upload"))
        })?;
        let scene_slug = pending.scene_slug.as_deref().ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("missing scene_slug in pending upload"))
        })?;
        let scene_dimension = pending.scene_dimension.as_deref().ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("missing scene_dimension in pending upload"))
        })?;

        sqlx::query_as!(
            crate::models::Scene,
            r#"
            INSERT INTO scenes (id, name, slug, description, dimension, active, created_at)
            VALUES ($1, $2, $3, $4, $5, TRUE, now())
            RETURNING *
            "#,
            id,
            scene_name,
            scene_slug,
            pending.scene_description,
            scene_dimension,
        )
        .fetch_one(&mut *tx)
        .await
        .context("failed to create scene")?;

        crate::id::SceneId(id)
    };

    // Create scene_version with package data
    let version_id = crate::id::generate_id();
    let version = SceneVersionRepo::create_with_package(
        &mut *tx,
        &version_id,
        scene_id.as_ref(),
        request.position.x,
        request.position.y,
        request.position.z,
        request.camera.pitch,
        request.camera.yaw,
        request.environment.time_of_day_ticks,
        &request.environment.weather,
        request.environment.weather_intensity,
        request.environment.moon_phase,
        request.biome.as_deref(),
        &pending.minecraft_version,
        &package_url,
        &pending.file_hash,
        pending.size_bytes,
        request.fov,
        request.render_distance,
    )
    .await?;

    // Auto-create default preset from environment data if this is the scene's first version
    let preset_id = if is_new_scene {
        let preset_id = ScenePresetId::generate();
        ScenePresetRepo::create(
            &mut *tx,
            preset_id.as_ref(),
            scene_id.as_ref(),
            "Default",
            "default",
            request.environment.time_of_day_ticks,
            &request.environment.weather,
            request.environment.weather_intensity,
            request.environment.moon_phase,
            0,
        )
        .await?;
        Some(preset_id)
    } else {
        None
    };

    // Delete the pending upload record
    sqlx::query!("DELETE FROM pending_scene_uploads WHERE id = $1", upload_id)
        .execute(&mut *tx)
        .await
        .context("failed to delete pending upload")?;

    tx.commit().await.context("failed to commit transaction")?;

    info!(
        scene_id = %scene_id,
        version_id = %version.id,
        is_new_scene,
        "Completed scene package upload"
    );

    Ok(Json(CompleteUploadResponse {
        scene_id,
        scene_version_id: version.id,
        preset_id,
    }))
}
