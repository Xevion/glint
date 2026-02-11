use std::collections::HashMap;
use std::net::SocketAddr;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    id::{self, ShaderVersionId},
    middleware::client_ip::ClientIp,
    models::{
        CaptureStatus, CreateShaderRequest, CreateShaderVersionRequest, Shader, ShaderListItem,
        ShaderVersion, ShaderWithCaptures, TrendingShader, UpdateShaderRequest,
    },
    repo::{
        CaptureRepo, CategoryRepo, FeatureRepo, ShaderAuthorRepo, ShaderRepo, ShaderVersionRepo,
        ShaderViewRepo,
        capture::{CaptureDistinct, CaptureFilters},
    },
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shaders).post(create_shader))
        .route("/trending", get(trending_shaders))
        .route(
            "/{id}",
            get(get_shader).put(update_shader).delete(delete_shader),
        )
        .route("/{id}/versions", post(create_shader_version))
}

/// Compute a viewer hash from client IP and User-Agent for deduplication.
fn viewer_hash(ip: &str, user_agent: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(b"|");
    hasher.update(user_agent.as_bytes());
    let result = hasher.finalize();
    // 16 hex chars = 8 bytes of entropy, plenty for hourly dedup
    hex::encode(&result[..8])
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
        authors_map
            .entry(a.shader_id.0.clone())
            .or_default()
            .push(a);
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
        .filter(|shader| thumbnails.contains_key(shader.id.as_ref()))
        .map(|shader| {
            let id = &shader.id;
            let id_str: &str = id.as_ref();
            let version = versions.get(id);
            let thumb = thumbnails.get(id_str);
            ShaderListItem {
                authors: authors_map.remove(id_str).unwrap_or_default(),
                categories: categories_map.remove(id_str).unwrap_or_default(),
                features: features_map.remove(id_str).unwrap_or_default(),
                latest_version: version.map(|v| v.version.clone()),
                game_versions: version.and_then(|v| v.game_versions.clone()),
                image_url: thumb.map(|t| t.image_url.clone()),
                thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
                shader,
            }
        })
        .collect();

    Ok(Json(items))
}

#[derive(Debug, Deserialize)]
struct ShaderDetailQuery {
    version_id: Option<String>,
    profile: Option<String>,
}

/// GET /api/shaders/{id} - Get shader by ID or slug with versions and captures (public)
async fn get_shader(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<String>,
    Query(query): Query<ShaderDetailQuery>,
    headers: HeaderMap,
) -> AppResult<Json<ShaderWithCaptures>> {
    let db = state.db();
    let shader = ShaderRepo::get(db, &id).await?;
    let versions = ShaderVersionRepo::list_by_shader_with_counts(db, shader.id.as_ref()).await?;

    // Default to latest version when no version_id is specified
    let effective_version_id = query
        .version_id
        .map(ShaderVersionId::from)
        .or_else(|| versions.first().map(|v| v.version.id.clone()));

    let filters = CaptureFilters {
        shader_id: Some(shader.id.clone()),
        version_id: effective_version_id,
        profile: query.profile,
        status: Some(CaptureStatus::Completed),
        scene_active: Some(true),
        ..Default::default()
    };
    let (captures, _) =
        CaptureRepo::list_with_context(db, &filters, None, CaptureDistinct::PerScene).await?;

    // Fire-and-forget view recording
    let hops = state.config().rate_limit.trusted_proxy_hops;
    let client_ip = ClientIp::resolve(&headers, Some(addr), hops);
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let hash = viewer_hash(&client_ip.0, ua);
    let shader_id = shader.id.0.clone();
    let pool = state.db().clone();
    tokio::spawn(async move {
        if let Err(e) = ShaderViewRepo::record_view(&pool, &shader_id, &hash).await {
            warn!(error = %e, shader_id, "Failed to record shader view");
        }
    });

    Ok(Json(ShaderWithCaptures {
        shader,
        versions,
        captures,
    }))
}

#[derive(Debug, Deserialize)]
struct TrendingQuery {
    /// Number of days to consider (default: 7)
    days: Option<i32>,
    /// Maximum number of results (default: 10)
    limit: Option<i64>,
}

/// GET /api/shaders/trending - Get trending shaders by recent view count (public)
async fn trending_shaders(
    State(state): State<AppState>,
    Query(query): Query<TrendingQuery>,
) -> AppResult<Json<Vec<TrendingShader>>> {
    let db = state.db();
    let days = query.days.unwrap_or(7).clamp(1, 90);
    let limit = query.limit.unwrap_or(10).clamp(1, 50);

    let trending = ShaderViewRepo::trending(db, days, limit).await?;

    if trending.is_empty() {
        return Ok(Json(vec![]));
    }

    let shader_ids: Vec<String> = trending.iter().map(|e| e.shader_id.0.clone()).collect();

    let (shaders_map, thumbnails) = tokio::try_join!(
        ShaderRepo::get_many(db, &shader_ids),
        CaptureRepo::batch_thumbnails_for_shaders(db, &shader_ids),
    )?;

    let result = trending
        .into_iter()
        .filter_map(|entry| {
            let id = entry.shader_id.as_ref();
            let shader = shaders_map.get(id).cloned();
            if shader.is_none() {
                tracing::debug!(shader_id = id, "Trending shader not found, skipping");
            }
            let thumb = thumbnails.get(id);
            shader.map(|s| TrendingShader {
                shader: s,
                trending_views: entry.view_count,
                image_url: thumb.map(|t| t.image_url.clone()),
                thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
            })
        })
        .collect();

    Ok(Json(result))
}

/// POST /api/shaders - Create a new shader (admin)
async fn create_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<CreateShaderRequest>,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let id = id::generate_id();
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

    let id = id::generate_id();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_hash_is_deterministic() {
        let h1 = viewer_hash("192.168.1.1", "Mozilla/5.0");
        let h2 = viewer_hash("192.168.1.1", "Mozilla/5.0");
        assert_eq!(h1, h2);
    }

    #[test]
    fn viewer_hash_differs_by_ip() {
        let h1 = viewer_hash("192.168.1.1", "Mozilla/5.0");
        let h2 = viewer_hash("10.0.0.1", "Mozilla/5.0");
        assert_ne!(h1, h2);
    }

    #[test]
    fn viewer_hash_differs_by_ua() {
        let h1 = viewer_hash("192.168.1.1", "Mozilla/5.0");
        let h2 = viewer_hash("192.168.1.1", "curl/7.88");
        assert_ne!(h1, h2);
    }

    #[test]
    fn viewer_hash_is_16_hex_chars() {
        let h = viewer_hash("1.2.3.4", "agent");
        assert_eq!(h.len(), 16, "should be 16 hex chars (8 bytes)");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
