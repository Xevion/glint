use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use tracing::instrument;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult, OptionNotFoundExt},
    models::{
        AdoptPreviewResponse, AdoptShaderRequest, LinkShaderRequest, Shader, ShaderSearchRequest,
        ShaderSearchResponse, ShaderSearchResult, ShaderSearchSort,
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

/// POST /api/shaders/search - Search or browse both platforms for shaders (admin)
///
/// When `query` is provided: searches platforms by text query.
/// When `query` is absent/empty: browses platforms using `sort` order (popular or recent).
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn search_platforms(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<ShaderSearchRequest>,
) -> AppResult<Json<ShaderSearchResponse>> {
    let query = request.query.as_deref().map(str::trim).unwrap_or_default();
    let limit = request.limit.unwrap_or(20).min(100);
    let offset = request.offset.unwrap_or(0);
    let sort = request.sort.unwrap_or_default();

    // Determine platform-specific sort parameters based on mode
    let (modrinth_index, cf_sort_field) = if query.is_empty() {
        match sort {
            ShaderSearchSort::Popular => (Some("downloads"), Some(6u32)), // TotalDownloads
            ShaderSearchSort::Recent => (Some("newest"), Some(3u32)),     // LastUpdated
        }
    } else {
        // For search queries, let platforms use their default relevance ranking
        (None, None)
    };

    let modrinth_fut = state
        .modrinth()
        .search_shaders(query, offset, limit, modrinth_index);
    let cf_fut = async {
        if let Some(cf) = state.curseforge() {
            Some(
                cf.search_shaders(query, limit, offset, cf_sort_field, Some("desc"))
                    .await,
            )
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
                    updated_at: Some(hit.date_modified),
                    adopted: None, // Enriched below
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
                let updated_at = cf_mod
                    .date_modified
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .ok();
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
                    updated_at,
                    adopted: None, // Enriched below
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

    // Sort combined results by downloads (for browse mode this maintains cross-platform ordering)
    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    // Enrich results with adopted shader info
    if !results.is_empty() {
        let modrinth_ids: Vec<String> = results
            .iter()
            .filter(|r| r.platform == "modrinth")
            .map(|r| r.platform_id.clone())
            .collect();
        let curseforge_ids: Vec<String> = results
            .iter()
            .filter(|r| r.platform == "curseforge")
            .map(|r| r.platform_id.clone())
            .collect();

        match ShaderRepo::find_adopted_by_platform_ids(state.db(), &modrinth_ids, &curseforge_ids)
            .await
        {
            Ok(adopted_map) => {
                for result in &mut results {
                    if let Some(adopted) = adopted_map.get(&result.platform_id) {
                        result.adopted = Some(adopted.clone());
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = ?e, "Failed to check adopted shaders, continuing without");
            }
        }
    }

    Ok(Json(ShaderSearchResponse {
        results,
        total_modrinth,
        total_curseforge,
    }))
}

/// POST /api/shaders/adopt/preview
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
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
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
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
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn link_platform(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
    Json(request): Json<LinkShaderRequest>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::find_by_id(state.db(), &shader_id)
        .await?
        .or_not_found("Shader", &shader_id)?;
    let platform_ref = platform::parse_platform_url(&request.url)
        .ok_or_else(|| AppError::BadRequest("Invalid or unsupported platform URL".into()))?;

    let shader = PlatformService::link_platform(&state, &shader, &platform_ref).await?;
    Ok(Json(shader))
}

/// POST /api/shaders/{id}/sync
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn sync_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::find_by_id(state.db(), &shader_id)
        .await?
        .or_not_found("Shader", &shader_id)?;
    let shader = PlatformService::sync_shader(&state, &shader).await?;
    Ok(Json(shader))
}
