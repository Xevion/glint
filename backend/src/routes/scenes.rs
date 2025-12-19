use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    error::{AppError, AppResult},
    models::{Capture, Scene, SceneWithCaptures},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_scenes))
        .route("/{slug}", get(get_scene))
}

async fn list_scenes(State(state): State<AppState>) -> AppResult<Json<Vec<Scene>>> {
    let scenes = sqlx::query_as::<_, Scene>("SELECT * FROM scenes ORDER BY name")
        .fetch_all(state.db())
        .await?;

    Ok(Json(scenes))
}

async fn get_scene(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<SceneWithCaptures>> {
    let scene = sqlx::query_as::<_, Scene>("SELECT * FROM scenes WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scene '{slug}' not found")))?;

    let captures = sqlx::query_as::<_, Capture>(
        "SELECT * FROM captures WHERE scene_id = ? AND status = 'completed'",
    )
    .bind(&scene.id)
    .fetch_all(state.db())
    .await?;

    Ok(Json(SceneWithCaptures { scene, captures }))
}
