use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use tracing::warn;

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::id::BackgroundId;
use crate::models::{
    Background, BackgroundUploadResponse, ConfirmBackgroundUploadRequest, UpdateBackgroundRequest,
};
use crate::repo::BackgroundRepo;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_enabled))
        .route("/all", get(list_all))
        .route("/", post(initiate_upload))
        .route("/{id}/confirm", post(confirm_upload))
        .route("/{id}", put(update_background))
        .route("/{id}", delete(delete_background))
}

/// GET /api/backgrounds — List enabled backgrounds (public, used by frontend SSR)
async fn list_enabled(State(state): State<AppState>) -> AppResult<Json<Vec<Background>>> {
    let backgrounds = BackgroundRepo::list_enabled(state.db()).await?;
    Ok(Json(backgrounds))
}

/// GET /api/backgrounds/all — List all backgrounds including disabled (admin)
async fn list_all(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<Background>>> {
    let backgrounds = BackgroundRepo::list_all(state.db()).await?;
    Ok(Json(backgrounds))
}

#[derive(Debug, Deserialize)]
pub struct InitiateUploadRequest {
    pub filename: String,
    pub content_type: Option<String>,
}

/// POST /api/backgrounds — Initiate a background upload (admin)
async fn initiate_upload(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<InitiateUploadRequest>,
) -> AppResult<Json<BackgroundUploadResponse>> {
    let s3 = state
        .s3()
        .ok_or_else(|| AppError::ServiceUnavailable("R2/S3 storage not configured".into()))?;

    let r2_config = &state.config().r2;
    let bucket = r2_config
        .bucket
        .as_deref()
        .ok_or_else(|| AppError::ServiceUnavailable("R2 bucket not configured".into()))?;

    let id = BackgroundId::generate();
    let content_type = request.content_type.as_deref().unwrap_or("image/webp");

    // Derive extension from content type
    let ext = match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/avif" => "avif",
        _ => "webp",
    };

    let r2_key = format!("backgrounds/{}.{}", id, ext);
    let image_url = r2_config.public_url_for_key(&r2_key);

    let presigned = s3
        .put_object()
        .bucket(bucket)
        .key(&r2_key)
        .content_type(content_type)
        .presigned(
            aws_sdk_s3::presigning::PresigningConfig::builder()
                .expires_in(std::time::Duration::from_secs(3600))
                .build()
                .expect("valid presigning config"),
        )
        .await
        .map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate presigned URL: {}", e))
        })?;

    // Create the DB record
    BackgroundRepo::insert(
        state.db(),
        id.as_ref(),
        &image_url,
        &r2_key,
        Some(&request.filename),
    )
    .await?;

    Ok(Json(BackgroundUploadResponse {
        id,
        presigned_url: presigned.uri().to_string(),
        image_url,
        image_path: r2_key,
    }))
}

/// POST /api/backgrounds/{id}/confirm — Confirm upload with metadata (admin)
async fn confirm_upload(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ConfirmBackgroundUploadRequest>,
) -> AppResult<Json<Background>> {
    let bg = BackgroundRepo::confirm_upload(
        state.db(),
        &id,
        request.thumbhash.as_deref(),
        request.width,
        request.height,
        request.file_size_bytes,
        request.content_type.as_deref(),
    )
    .await?;
    Ok(Json(bg))
}

/// PUT /api/backgrounds/{id} — Update background metadata (admin)
async fn update_background(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateBackgroundRequest>,
) -> AppResult<Json<Background>> {
    let theme_str = request.theme_mode.as_ref().map(|t| t.as_str());
    let bg = BackgroundRepo::update(
        state.db(),
        &id,
        request.enabled,
        theme_str,
        request.sort_order,
    )
    .await?;
    Ok(Json(bg))
}

/// DELETE /api/backgrounds/{id} — Delete a background and its R2 file (admin)
async fn delete_background(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<()>> {
    let image_path = BackgroundRepo::delete(state.db(), &id).await?;

    // Best-effort R2 cleanup
    if let (Some(path), Some(s3)) = (image_path, state.s3()) {
        let bucket = state.config().r2.bucket.as_deref().unwrap_or("glint");
        if let Err(e) = s3.delete_object().bucket(bucket).key(&path).send().await {
            warn!(path, error = %e, "Failed to delete background from R2");
        }
    }

    Ok(Json(()))
}
