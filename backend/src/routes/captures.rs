use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{Capture, CaptureWithContext},
    repo::CaptureRepo,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_captures_public))
        .route("/all", get(list_captures_all))
        .route("/{id}", get(get_capture_public).delete(delete_capture))
        .route("/{id}/details", get(get_capture_details))
}

/// GET /api/captures - List completed captures (public)
async fn list_captures_public(State(state): State<AppState>) -> AppResult<Json<Vec<Capture>>> {
    let captures = CaptureRepo::list_completed(state.db()).await?;
    Ok(Json(captures))
}

/// GET /api/captures/all - List all captures with context (admin)
async fn list_captures_all(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<CaptureWithContext>>> {
    let captures = CaptureRepo::list_all_with_context(state.db()).await?;
    Ok(Json(captures))
}

/// GET /api/captures/{id} - Get capture by ID (public)
async fn get_capture_public(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Capture>> {
    let capture = CaptureRepo::get_by_id(state.db(), &id).await?;
    Ok(Json(capture))
}

/// GET /api/captures/{id}/details - Get capture with context (admin)
async fn get_capture_details(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<CaptureWithContext>> {
    let capture = CaptureRepo::get_with_context(state.db(), &id).await?;
    Ok(Json(capture))
}

/// DELETE /api/captures/{id} - Delete a capture (admin)
async fn delete_capture(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = CaptureRepo::delete(state.db(), &id).await?;
    if !deleted {
        return Err(AppError::NotFound("Capture not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}
