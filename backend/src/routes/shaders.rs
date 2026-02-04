use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use tracing::info;
use uuid::Uuid;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{
        CreateShaderRequest, CreateShaderVersionRequest, Shader, ShaderVersion, ShaderWithCaptures,
        UpdateShaderRequest,
    },
    repo::{ShaderRepo, ShaderVersionRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shaders).post(create_shader))
        .route(
            "/{id}",
            get(get_shader_by_id)
                .put(update_shader)
                .delete(delete_shader),
        )
        .route("/by-slug/{slug}", get(get_shader_by_slug))
        .route("/{id}/versions", post(create_shader_version))
}

/// GET /api/shaders - List all shaders (public)
async fn list_shaders(State(state): State<AppState>) -> AppResult<Json<Vec<Shader>>> {
    let shaders = ShaderRepo::list(state.db()).await?;
    Ok(Json(shaders))
}

/// GET /api/shaders/by-slug/{slug} - Get shader by slug with captures (public)
async fn get_shader_by_slug(
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

/// GET /api/shaders/{id} - Get shader by ID (admin)
async fn get_shader_by_id(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::get_by_id(state.db(), &id).await?;
    Ok(Json(shader))
}

/// POST /api/shaders - Create a new shader (admin)
async fn create_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<CreateShaderRequest>,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let id = Uuid::new_v4().to_string();
    let shader = ShaderRepo::create(state.db(), &id, &request).await?;
    info!(shader_id = %shader.id, slug = %shader.slug, "Shader created");
    Ok((StatusCode::CREATED, Json(shader)))
}

/// POST /api/shaders/{id}/versions - Create a new shader version (admin)
async fn create_shader_version(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
    Json(request): Json<CreateShaderVersionRequest>,
) -> AppResult<(StatusCode, Json<ShaderVersion>)> {
    if !ShaderRepo::exists_by_id(state.db(), &shader_id).await? {
        return Err(AppError::NotFound("Shader not found".into()));
    }

    let id = Uuid::new_v4().to_string();
    let version = ShaderVersionRepo::create(state.db(), &id, &shader_id, &request).await?;

    Ok((StatusCode::CREATED, Json(version)))
}

/// PUT /api/shaders/{id} - Update shader metadata (admin)
async fn update_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateShaderRequest>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::update(state.db(), &id, &request).await?;
    Ok(Json(shader))
}

/// DELETE /api/shaders/{id} - Delete a shader (admin)
async fn delete_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = ShaderRepo::delete(state.db(), &id).await?;
    if !deleted {
        return Err(AppError::NotFound("Shader not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}
