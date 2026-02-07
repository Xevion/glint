use axum::{Json, Router, extract::State, routing::get};

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::repo::CaptureHealthRepo;
use crate::repo::capture_health::CaptureHealthResponse;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_capture_health))
}

/// GET /api/admin/capture-health — Full capture target matrix with health status
async fn get_capture_health(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<CaptureHealthResponse>> {
    let health = CaptureHealthRepo::get_capture_health(state.db()).await?;
    Ok(Json(health))
}
