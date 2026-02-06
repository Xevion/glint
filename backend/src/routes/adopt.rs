use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{
        AdoptPreviewResponse, AdoptShaderRequest, LinkShaderRequest, Shader, ShaderSearchRequest,
        ShaderSearchResponse, ShaderSearchResult,
    },
    platform::{self, Platform},
    repo::ShaderRepo,
    services::platform::PlatformService,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/adopt/preview", post(adopt_preview))
        .route("/adopt", post(adopt))
        .route("/search", post(search_platforms))
        .route("/{id}/link", post(link_platform))
        .route("/{id}/sync", post(sync_shader))
}

/// POST /api/shaders/search - Search both platforms for shaders (admin)
async fn search_platforms(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<ShaderSearchRequest>,
) -> AppResult<Json<ShaderSearchResponse>> {
    let query = request.query.trim();
    if query.is_empty() {
        return Err(AppError::BadRequest("Search query cannot be empty".into()));
    }

    let limit = request.limit.unwrap_or(20).min(100);
    let offset = request.offset.unwrap_or(0);

    let modrinth_fut = state.modrinth().search_shaders(query, offset, limit);
    let cf_fut = async {
        if let Some(cf) = state.curseforge() {
            Some(cf.search_shaders(query, limit, offset).await)
        } else {
            None
        }
    };

    let (modrinth_result, cf_result) = tokio::join!(modrinth_fut, cf_fut);

    let mut results: Vec<ShaderSearchResult> = Vec::new();
    let total_modrinth;

    match modrinth_result {
        Ok(search) => {
            total_modrinth = search.total_hits;
            for hit in search.hits {
                let slug = hit.slug.unwrap_or_default();
                results.push(ShaderSearchResult {
                    platform: Platform::Modrinth.to_string(),
                    platform_id: hit.project_id,
                    platform_url: format!("https://modrinth.com/shader/{slug}"),
                    slug,
                    name: hit.title,
                    description: hit.description,
                    icon_url: hit.icon_url,
                    author: hit.author,
                    downloads: hit.downloads,
                    categories: hit.categories,
                });
            }
        }
        Err(e) => {
            tracing::warn!(error = ?e, "Modrinth search failed");
            total_modrinth = 0;
        }
    }

    let total_curseforge = match cf_result {
        Some(Ok(search)) => {
            let total = search.pagination.map(|p| p.total_count);
            for cf_mod in search.data {
                results.push(ShaderSearchResult {
                    platform: Platform::CurseForge.to_string(),
                    platform_id: cf_mod.id.to_string(),
                    slug: cf_mod.slug.clone(),
                    name: cf_mod.name,
                    description: cf_mod.summary,
                    icon_url: cf_mod.logo.map(|l| l.url),
                    author: cf_mod
                        .authors
                        .first()
                        .map(|a| a.name.clone())
                        .unwrap_or_default(),
                    downloads: cf_mod.download_count,
                    categories: cf_mod.categories.into_iter().map(|c| c.name).collect(),
                    platform_url: format!(
                        "https://www.curseforge.com/minecraft/shaders/{}",
                        cf_mod.slug
                    ),
                });
            }
            total
        }
        Some(Err(e)) => {
            tracing::warn!(error = ?e, "CurseForge search failed");
            Some(0)
        }
        None => None,
    };

    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    Ok(Json(ShaderSearchResponse {
        results,
        total_modrinth,
        total_curseforge,
    }))
}

/// POST /api/shaders/adopt/preview
async fn adopt_preview(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<AdoptShaderRequest>,
) -> AppResult<Json<AdoptPreviewResponse>> {
    let platform_ref = platform::parse_platform_url(&request.url)
        .ok_or_else(|| AppError::BadRequest("Invalid or unsupported platform URL".into()))?;

    let preview = PlatformService::preview_shader(&state, &platform_ref).await?;
    Ok(Json(preview))
}

/// POST /api/shaders/adopt
async fn adopt(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<AdoptShaderRequest>,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let platform_ref = platform::parse_platform_url(&request.url)
        .ok_or_else(|| AppError::BadRequest("Invalid or unsupported platform URL".into()))?;

    let shader = PlatformService::adopt_shader(&state, &platform_ref).await?;
    Ok((StatusCode::CREATED, Json(shader)))
}

/// POST /api/shaders/{id}/link
async fn link_platform(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
    Json(request): Json<LinkShaderRequest>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;
    let platform_ref = platform::parse_platform_url(&request.url)
        .ok_or_else(|| AppError::BadRequest("Invalid or unsupported platform URL".into()))?;

    let shader = PlatformService::link_platform(&state, &shader, &platform_ref).await?;
    Ok(Json(shader))
}

/// POST /api/shaders/{id}/sync
async fn sync_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;
    let shader = PlatformService::sync_shader(&state, &shader).await?;
    Ok(Json(shader))
}
