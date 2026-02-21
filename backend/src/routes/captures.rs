use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use tracing::{instrument, warn};

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult, OptionNotFoundExt},
    models::CaptureDetail,
    repo::{CaptureRepo, SceneRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(get_capture_public).delete(delete_capture))
        .route("/{id}/details", get(get_capture_details))
}

/// Check `If-None-Match` against an ETag value.
fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|t| t.trim().trim_matches('"') == etag.trim_matches('"'))
        })
}

/// GET /api/captures/{id} - Get capture by ID (public, only from active scenes)
#[instrument(skip(state, headers))]
async fn get_capture_public(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let capture = CaptureRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("Capture", &id)?;

    // Verify the capture's scene is active (don't leak disabled-scene captures)
    let scene = SceneRepo::find_by_id(state.db(), capture.scene_id.as_ref()).await?;
    match scene {
        Some(s) if s.active => {}
        _ => return Err(AppError::NotFound(format!("Capture '{}' not found", id))),
    }

    let etag = format!("\"c:{}:{}\"", capture.id, capture.updated_at.timestamp());

    if etag_matches(&headers, &etag) {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        if let Ok(val) = HeaderValue::from_str(&etag) {
            response.headers_mut().insert(header::ETAG, val);
        }
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static(
                "public, max-age=120, s-maxage=120, stale-while-revalidate=600",
            ),
        );
        return Ok(response);
    }

    let mut response = Json(capture).into_response();
    if let Ok(val) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(header::ETAG, val);
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=120, s-maxage=120, stale-while-revalidate=600"),
    );
    Ok(response)
}

/// GET /api/captures/{id}/details - Get full capture detail with related captures (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn get_capture_details(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<CaptureDetail>> {
    let detail = CaptureRepo::get_detail(state.db(), &id).await?;
    Ok(Json(detail))
}

/// DELETE /api/captures/{id} - Delete a capture and its R2 image (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn delete_capture(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let capture = CaptureRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("Capture", &id)?;

    let deleted = CaptureRepo::delete(state.db(), &id).await?;
    if !deleted {
        return Err(AppError::NotFound("Capture not found".into()));
    }

    // Best-effort R2 cleanup
    if let Some(s3) = state.s3() {
        let bucket = state.config().r2.bucket.as_deref().unwrap_or("glint");
        if let Err(e) = s3
            .delete_object()
            .bucket(bucket)
            .key(&capture.image_path)
            .send()
            .await
        {
            warn!(image_path = capture.image_path, error = %e, "Failed to delete capture image from R2");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
