use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    error::AppResult,
    models::ShaderWithCaptures,
    repo::{ShaderRepo, ShaderVersionRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shaders))
        .route("/{slug}", get(get_shader))
}

async fn list_shaders(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<crate::models::Shader>>> {
    let shaders = ShaderRepo::list(state.db()).await?;
    Ok(Json(shaders))
}

async fn get_shader(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<ShaderWithCaptures>> {
    let shader = ShaderRepo::get_by_slug(state.db(), &slug).await?;
    let versions = ShaderVersionRepo::list_by_shader(state.db(), &shader.id).await?;
    let captures = ShaderRepo::get_captures_with_context(state.db(), &shader.id).await?;

    Ok(Json(ShaderWithCaptures {
        shader,
        versions,
        captures,
    }))
}
