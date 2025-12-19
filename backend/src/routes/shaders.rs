use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    error::{AppError, AppResult},
    models::{Capture, Shader, ShaderWithCaptures},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shaders))
        .route("/{slug}", get(get_shader))
}

async fn list_shaders(State(state): State<AppState>) -> AppResult<Json<Vec<Shader>>> {
    let shaders = sqlx::query_as::<_, Shader>("SELECT * FROM shaders ORDER BY name")
        .fetch_all(state.db())
        .await?;

    Ok(Json(shaders))
}

async fn get_shader(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<ShaderWithCaptures>> {
    let shader = sqlx::query_as::<_, Shader>("SELECT * FROM shaders WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Shader '{slug}' not found")))?;

    let captures = sqlx::query_as::<_, Capture>(
        "SELECT * FROM captures WHERE shader_id = ? AND status = 'completed'",
    )
    .bind(&shader.id)
    .fetch_all(state.db())
    .await?;

    Ok(Json(ShaderWithCaptures { shader, captures }))
}
