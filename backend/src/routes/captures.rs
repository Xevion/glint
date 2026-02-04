use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{error::AppResult, models::Capture, repo::CaptureRepo, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_captures))
        .route("/{id}", get(get_capture))
}

async fn list_captures(State(state): State<AppState>) -> AppResult<Json<Vec<Capture>>> {
    let captures = CaptureRepo::list_completed(state.db()).await?;
    Ok(Json(captures))
}

async fn get_capture(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Capture>> {
    let capture = CaptureRepo::get_by_id(state.db(), &id).await?;
    Ok(Json(capture))
}
