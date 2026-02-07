use std::collections::HashMap;

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
        CreateShaderRequest, CreateShaderVersionRequest, Shader, ShaderListItem, ShaderVersion,
        ShaderWithCaptures, UpdateShaderRequest,
    },
    repo::{
        CaptureRepo, CategoryRepo, FeatureRepo, ShaderAuthorRepo, ShaderRepo, ShaderVersionRepo,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shaders).post(create_shader))
        .route(
            "/{id}",
            get(get_shader).put(update_shader).delete(delete_shader),
        )
        .route("/{id}/versions", post(create_shader_version))
}

/// GET /api/shaders - List all shaders with enrichment (public)
async fn list_shaders(State(state): State<AppState>) -> AppResult<Json<Vec<ShaderListItem>>> {
    let db = state.db();

    let (shaders, authors, categories, features, versions, thumbnails) = tokio::try_join!(
        ShaderRepo::list(db),
        ShaderAuthorRepo::list_all(db),
        CategoryRepo::list_all_for_shaders(db),
        FeatureRepo::list_all_for_shaders(db),
        ShaderVersionRepo::batch_latest_versions(db),
        CaptureRepo::batch_thumbnails_by_shader(db),
    )?;

    // Group by shader_id
    let mut authors_map: HashMap<String, Vec<_>> = HashMap::new();
    for a in authors {
        authors_map.entry(a.shader_id.clone()).or_default().push(a);
    }

    let mut categories_map: HashMap<String, Vec<_>> = HashMap::new();
    for (sid, cat) in categories {
        categories_map.entry(sid).or_default().push(cat);
    }

    let mut features_map: HashMap<String, Vec<_>> = HashMap::new();
    for (sid, feat) in features {
        features_map.entry(sid).or_default().push(feat);
    }

    let items = shaders
        .into_iter()
        .filter(|shader| thumbnails.contains_key(&shader.id))
        .map(|shader| {
            let id = &shader.id;
            let version = versions.get(id);
            let thumb = thumbnails.get(id);
            ShaderListItem {
                authors: authors_map.remove(id).unwrap_or_default(),
                categories: categories_map.remove(id).unwrap_or_default(),
                features: features_map.remove(id).unwrap_or_default(),
                latest_version: version.map(|v| v.version.clone()),
                game_versions: version.and_then(|v| v.game_versions.clone()),
                thumbnail_url: thumb.map(|t| t.image_url.clone()),
                thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
                shader,
            }
        })
        .collect();

    Ok(Json(items))
}

/// GET /api/shaders/{id} - Get shader by ID or slug with versions and captures (public)
async fn get_shader(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ShaderWithCaptures>> {
    let shader = ShaderRepo::get(state.db(), &id).await?;
    let versions = ShaderVersionRepo::list_by_shader(state.db(), &shader.id).await?;
    let captures = ShaderRepo::get_captures_with_context(state.db(), &shader.id).await?;

    Ok(Json(ShaderWithCaptures {
        shader,
        versions,
        captures,
    }))
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
