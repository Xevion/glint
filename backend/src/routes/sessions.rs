use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::delete,
};

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    repo::SessionRepo,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/{token}", delete(delete_session))
}

/// DELETE /api/sessions/{token} - Delete a specific session (admin)
async fn delete_session(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = SessionRepo::delete(state.db(), &token).await?;
    if !deleted {
        return Err(AppError::NotFound("Session not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}
