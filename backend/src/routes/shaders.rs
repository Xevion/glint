use std::net::SocketAddr;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult, OptionNotFoundExt},
    id::{self, ShaderVersionId},
    middleware::client_ip::ClientIp,
    models::pagination::normalize_pagination,
    models::{
        CaptureStatus, CreateShaderRequest, CreateShaderVersionRequest, Paginated, Shader,
        ShaderListItem, ShaderVersion, ShaderWithCaptures, UpdateShaderRequest,
    },
    repo::{
        CaptureRepo, ShaderRepo, ShaderVersionRepo, ShaderViewRepo, SlugRedirectRepo,
        capture::{CaptureDistinct, CaptureFilters},
    },
    services::shader::ShaderService,
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::{info, instrument, warn};

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

/// Response type for shader detail endpoint: JSON data, 301 redirect, or 304 not modified.
enum ShaderDetailResponse {
    Data {
        body: Box<Json<ShaderWithCaptures>>,
        etag: String,
    },
    Redirect(Redirect),
    NotModified(String),
}

impl IntoResponse for ShaderDetailResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Data { body, etag } => {
                let mut response = body.into_response();
                if let Ok(val) = HeaderValue::from_str(&etag) {
                    response.headers_mut().insert(header::ETAG, val);
                }
                response.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static(
                        "public, max-age=30, s-maxage=30, stale-while-revalidate=120",
                    ),
                );
                response
            }
            Self::Redirect(r) => r.into_response(),
            Self::NotModified(etag) => {
                let mut response = StatusCode::NOT_MODIFIED.into_response();
                if let Ok(val) = HeaderValue::from_str(&etag) {
                    response.headers_mut().insert(header::ETAG, val);
                }
                response.headers_mut().insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static(
                        "public, max-age=30, s-maxage=30, stale-while-revalidate=120",
                    ),
                );
                response
            }
        }
    }
}

/// Build a redirect URL for a shader, preserving query params.
fn build_shader_redirect_url(slug: &str, query: &ShaderDetailQuery) -> String {
    let mut url = format!("/api/shaders/{}", slug);
    let mut params = Vec::new();
    if let Some(ref v) = query.version_id {
        params.push(format!("versionId={v}"));
    }
    if let Some(ref p) = query.profile {
        params.push(format!("profile={p}"));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
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

#[derive(Debug, Deserialize)]
struct ShaderListQuery {
    page: Option<i32>,
    page_size: Option<i32>,
}

/// GET /api/shaders - Paginated list of shaders with enrichment (public)
#[instrument(skip(state))]
async fn list_shaders(
    State(state): State<AppState>,
    Query(params): Query<ShaderListQuery>,
) -> AppResult<Json<Paginated<ShaderListItem>>> {
    let p = normalize_pagination(params.page, params.page_size);
    let (items, total) =
        ShaderService::list_enriched(state.db(), p.page_size as i64, p.offset).await?;

    Ok(Json(Paginated {
        items,
        total,
        page: p.page,
        page_size: p.page_size,
    }))
}

#[derive(Debug, Deserialize)]
struct ShaderDetailQuery {
    version_id: Option<String>,
    profile: Option<String>,
}

/// Check `If-None-Match` against an ETag value; return `true` if the client's
/// cached version matches (meaning we can return 304).
fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|t| t.trim().trim_matches('"') == etag.trim_matches('"'))
        })
}

/// GET /api/shaders/{id} - Get shader by ID or slug with versions and captures (public)
///
/// Supports 301 redirects: if the path param is a nanoid, old slug, or non-canonical slug,
/// the response redirects to the canonical `/api/shaders/{current_slug}` URL.
#[instrument(skip(state, headers))]
async fn get_shader(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<String>,
    Query(query): Query<ShaderDetailQuery>,
    headers: HeaderMap,
) -> Result<ShaderDetailResponse, AppError> {
    let db = state.db();

    // Try lookup by ID or slug without auto-404
    let shader = if id.len() <= crate::id::ID_LENGTH {
        // Could be a nanoid or a short slug — check both
        ShaderRepo::find_by_id(db, &id)
            .await?
            .or(ShaderRepo::find_by_slug(db, &id).await?)
    } else {
        ShaderRepo::find_by_slug(db, &id).await?
    };

    let shader = match shader {
        Some(shader) => {
            if id != shader.slug {
                // Input was nanoid or non-canonical — redirect to canonical slug URL
                let url = build_shader_redirect_url(&shader.slug, &query);
                return Ok(ShaderDetailResponse::Redirect(Redirect::permanent(&url)));
            }
            shader
        }
        None => {
            // Not found by ID or slug — check redirect table for old slugs
            if let Some(entity_id) = SlugRedirectRepo::find_entity_id(db, "shader", &id).await?
                && let Some(shader) = ShaderRepo::find_by_id(db, &entity_id).await?
            {
                let url = build_shader_redirect_url(&shader.slug, &query);
                return Ok(ShaderDetailResponse::Redirect(Redirect::permanent(&url)));
            }
            return Err(AppError::NotFound(format!("Shader '{}' not found", id)));
        }
    };

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

    // ETag based on shader identity + update timestamp
    let etag = format!("\"s:{}:{}\"", shader.id, shader.updated_at.timestamp());

    if etag_matches(&headers, &etag) {
        return Ok(ShaderDetailResponse::NotModified(etag));
    }

    Ok(ShaderDetailResponse::Data {
        body: Box::new(Json(ShaderWithCaptures {
            shader,
            versions,
            captures,
        })),
        etag,
    })
}

#[derive(Debug, Deserialize)]
struct TrendingQuery {
    /// Number of days to consider (default: 7)
    days: Option<i32>,
    /// Maximum number of results (default: 10)
    limit: Option<i64>,
}

/// GET /api/shaders/trending - Get trending shaders by recent view count (public)
#[instrument(skip(state))]
async fn trending_shaders(
    State(state): State<AppState>,
    Query(query): Query<TrendingQuery>,
) -> AppResult<Response> {
    let days = query.days.unwrap_or(7).clamp(1, 90);
    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let result = ShaderService::list_trending(state.db(), days, limit).await?;
    let mut response = Json(result).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300, s-maxage=300, stale-while-revalidate=600"),
    );
    Ok(response)
}

/// POST /api/shaders - Create a new shader (admin)
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn create_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<CreateShaderRequest>,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let slug = request
        .slug
        .clone()
        .unwrap_or_else(|| crate::slug::slugify(&request.name));
    let id = id::generate_id();
    // Clear any old redirect for this slug so the new entity owns it cleanly
    SlugRedirectRepo::delete_by_old_slug(state.db(), "shader", &slug).await?;
    let shader = ShaderRepo::create(state.db(), &id, &slug, &request).await?;
    info!(shader_id = %shader.id, slug = %shader.slug, "Shader created");
    Ok((StatusCode::CREATED, Json(shader)))
}

/// POST /api/shaders/{id}/versions - Create a new shader version (admin)
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
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
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn update_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateShaderRequest>,
) -> AppResult<Json<Shader>> {
    // If slug is being changed, record the old one as a redirect
    if let Some(ref new_slug) = request.slug {
        let old = ShaderRepo::find_by_id(state.db(), &id)
            .await?
            .or_not_found("Shader", &id)?;
        if old.slug != *new_slug {
            SlugRedirectRepo::upsert(state.db(), "shader", &old.slug, &old.id.0).await?;
            // Clear any existing redirect for the new slug so it's not stale
            SlugRedirectRepo::delete_by_old_slug(state.db(), "shader", new_slug).await?;
        }
    }
    let shader = ShaderRepo::update(state.db(), &id, &request).await?;
    Ok(Json(shader))
}

/// DELETE /api/shaders/{id} - Delete a shader (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
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
