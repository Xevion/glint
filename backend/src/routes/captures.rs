use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    error::{AppError, AppResult},
    models::Capture,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_captures))
        .route("/{id}", get(get_capture))
}

async fn list_captures(State(state): State<AppState>) -> AppResult<Json<Vec<Capture>>> {
    let captures = sqlx::query_as!(
        Capture,
        "SELECT * FROM captures WHERE status = 'completed' ORDER BY created_at DESC"
    )
    .fetch_all(state.db())
    .await?;

    Ok(Json(captures))
}

async fn get_capture(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Capture>> {
    let capture = sqlx::query_as!(Capture, "SELECT * FROM captures WHERE id = $1", id)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Capture '{id}' not found")))?;

    Ok(Json(capture))
}
