use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use serde::Serialize;
use tracing::{info, instrument};

use crate::auth::AdminUser;
use crate::error::{AppError, AppResult};
use crate::repo::{ExtractionRepo, ShaderVersionRepo};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/{version_id}/extract", post(trigger_extraction))
}

#[derive(Serialize)]
struct ExtractResponse {
    version_id: String,
    extraction_status: &'static str,
}

/// POST /api/admin/shaders/{version_id}/extract — Trigger re-extraction for a shader version
///
/// Resets `extraction_status` to `'pending'` so the background extraction service
/// picks it up again. Useful when extraction failed or a shader pack was updated.
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn trigger_extraction(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(version_id): Path<String>,
) -> AppResult<(StatusCode, Json<ExtractResponse>)> {
    if !ShaderVersionRepo::exists_by_id(state.db(), &version_id).await? {
        return Err(AppError::NotFound(format!(
            "Shader version '{version_id}' not found"
        )));
    }

    ExtractionRepo::reset_extraction_status(state.db(), &version_id).await?;

    info!(version_id, "Triggered re-extraction for shader version");

    Ok((
        StatusCode::ACCEPTED,
        Json(ExtractResponse {
            version_id,
            extraction_status: "pending",
        }),
    ))
}
