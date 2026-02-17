use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, put},
};
use custom_debug_derive::Debug as CustomDebug;
use serde::Deserialize;
use tracing::instrument;
use validator::Validate;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult, OptionNotFoundExt},
    models::{
        CaptureStatus, CaptureWithContext, CreateSceneRequest, PageQuery, Paginated, Scene,
        SceneListItem, SceneWithCaptures, SceneWithVersion, SceneWithWorld,
        UpdateSceneMetadataRequest, UpdateSceneRequest,
    },
    repo::{
        CaptureRepo, SceneRepo, SceneVersionRepo, SlugRedirectRepo, TagRepo, WorldRepo,
        capture::{CaptureDistinct, CaptureFilters},
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_scenes_public).post(create_scene))
        .route("/all", get(list_scenes_all))
        .route("/batch", delete(batch_disable_scenes))
        .route(
            "/{id}",
            get(get_scene_by_id)
                .put(update_scene_metadata)
                .delete(disable_scene_by_id),
        )
        .route("/{id}/reactivate", put(reactivate_scene))
        .route(
            "/by-slug/{slug}",
            get(get_scene_by_slug)
                .put(update_scene)
                .delete(disable_scene),
        )
        .route("/by-slug/{slug}/captures", get(list_scene_captures))
}

/// GET /api/scenes - List active scenes with enrichment (public), optionally filtered by world_id
#[instrument(skip(state))]
async fn list_scenes_public(
    State(state): State<AppState>,
    Query(params): Query<SceneQuery>,
) -> AppResult<Json<Vec<SceneListItem>>> {
    let db = state.db();

    let scenes = if let Some(ref world_id) = params.world_id {
        SceneRepo::list_by_world(db, world_id).await?
    } else {
        SceneRepo::list_active(db).await?
    };

    let (tags, thumbnails, counts) = tokio::try_join!(
        TagRepo::list_all_for_scenes(db),
        CaptureRepo::batch_thumbnails_by_scene(db),
        CaptureRepo::batch_count_by_scene(db),
    )?;

    let mut tags_map: HashMap<String, Vec<_>> = HashMap::new();
    for (sid, tag) in tags {
        tags_map.entry(sid).or_default().push(tag);
    }

    // Fetch latest version for each scene (single batch query)
    let scene_ids: Vec<String> = scenes.iter().map(|s| s.id.0.clone()).collect();
    let mut version_map = SceneVersionRepo::get_latest_batch(db, &scene_ids).await?;

    let items: Vec<SceneListItem> = scenes
        .into_iter()
        .map(|scene| {
            let id = &scene.id;
            let id_str: &str = id.as_ref();
            let thumb = thumbnails.get(id_str);
            let version = version_map.remove(id).ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("scene '{}' has no version", id))
            })?;
            Ok(SceneListItem {
                tags: tags_map.remove(id_str).unwrap_or_default(),
                image_url: thumb.map(|t| t.image_url.clone()),
                thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
                capture_count: counts.get(id_str).copied().unwrap_or(0),
                version,
                scene,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(Json(items))
}

/// GET /api/scenes/all - List all scenes including disabled (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn list_scenes_all(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SceneWithWorld>>> {
    let scenes = SceneRepo::list_all(state.db()).await?;
    Ok(Json(scenes))
}

#[derive(CustomDebug, Deserialize)]
struct SceneQuery {
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    world_id: Option<String>,
}

/// Response type for scene-by-slug endpoint: JSON data, 301 redirect, or 304 not modified.
enum SceneSlugResponse {
    Data {
        body: Json<Vec<SceneWithCaptures>>,
        etag: String,
    },
    Redirect(Redirect),
    NotModified(String),
}

impl IntoResponse for SceneSlugResponse {
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

/// Check `If-None-Match` against an ETag value.
fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|t| t.trim().trim_matches('"') == etag.trim_matches('"'))
        })
}

/// GET /api/scenes/by-slug/{slug} - Get scene by slug with captures (public)
///
/// Supports 301 redirects when accessing via an old slug that has since been renamed.
#[instrument(skip(state, headers))]
async fn get_scene_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<SceneQuery>,
    headers: HeaderMap,
) -> Result<SceneSlugResponse, AppError> {
    // Fetch all scenes with this slug (world-scoped), optionally filtered by world_id
    let scenes = if let Some(ref world_id) = params.world_id {
        match SceneRepo::find_by_slug_and_world(state.db(), &slug, world_id).await? {
            Some(scene) => vec![scene],
            None => vec![],
        }
    } else {
        SceneRepo::find_active_by_slug(state.db(), &slug).await?
    };

    // If no scenes found, check redirect table before returning 404
    if scenes.is_empty() {
        if let Some(entity_id) =
            SlugRedirectRepo::find_entity_id(state.db(), "scene", &slug).await?
            && let Some(scene) = SceneRepo::find_by_id(state.db(), &entity_id).await?
        {
            let mut url = format!("/api/scenes/by-slug/{}", scene.slug);
            if let Some(ref wid) = params.world_id {
                url.push_str(&format!("?worldId={wid}"));
            }
            return Ok(SceneSlugResponse::Redirect(Redirect::permanent(&url)));
        }
        return Err(AppError::NotFound(format!("Scene '{}' not found", slug)));
    }

    // Build response with captures for each scene
    let mut results = Vec::new();
    for scene in scenes {
        let scene_filters = CaptureFilters {
            scene_id: Some(scene.id.clone()),
            status: Some(CaptureStatus::Completed),
            ..Default::default()
        };
        let (world, version, (captures, _)) = tokio::try_join!(
            WorldRepo::find_by_id(state.db(), scene.world_id.as_ref()),
            SceneVersionRepo::get_latest(state.db(), scene.id.as_ref()),
            CaptureRepo::list_with_context(
                state.db(),
                &scene_filters,
                None,
                CaptureDistinct::PerShader,
            ),
        )?;

        let version = version.ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("scene '{}' has no version", scene.id))
        })?;

        results.push(SceneWithCaptures {
            scene,
            version,
            world,
            captures,
        });
    }

    // ETag from the latest scene version timestamp across all results
    let max_ts = results
        .iter()
        .map(|r| r.version.created_at.timestamp())
        .max()
        .unwrap_or(0);
    let etag = format!("\"sc:{}:{}\"", slug, max_ts);

    if etag_matches(&headers, &etag) {
        return Ok(SceneSlugResponse::NotModified(etag));
    }

    Ok(SceneSlugResponse::Data {
        body: Json(results),
        etag,
    })
}

/// GET /api/scenes/{id} - Get scene by ID with latest version (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn get_scene_by_id(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<SceneWithVersion>> {
    let scene = SceneRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("Scene", &id)?;
    let version = SceneVersionRepo::get_latest(state.db(), &id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("scene '{}' has no version", id)))?;
    Ok(Json(SceneWithVersion { scene, version }))
}

/// POST /api/scenes - Create a new scene (admin)
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn create_scene(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<CreateSceneRequest>,
) -> AppResult<(StatusCode, Json<SceneWithVersion>)> {
    request.validate()?;

    // Verify world exists
    if !WorldRepo::exists_by_id(state.db(), request.world_id.as_ref()).await? {
        return Err(AppError::NotFound("World not found".into()));
    }

    // Check world-scoped slug uniqueness (only active scenes)
    if SceneRepo::exists_by_slug_in_world(state.db(), &request.slug, request.world_id.as_ref())
        .await?
    {
        return Err(AppError::Conflict(format!(
            "Scene with slug '{}' already exists in this world",
            request.slug
        )));
    }

    let id = crate::id::generate_id();
    let (scene, version) = SceneRepo::create(state.db(), &id, &request).await?;

    Ok((
        StatusCode::CREATED,
        Json(SceneWithVersion { scene, version }),
    ))
}

/// PUT /api/scenes/by-slug/{slug} - Update scene by slug (admin)
#[derive(Debug, Deserialize)]
struct WorldIdParam {
    world_id: String,
}

#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn update_scene(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateSceneRequest>,
) -> AppResult<Json<SceneWithVersion>> {
    request.validate()?;

    // Find scene (world-scoped, active only)
    let scene = SceneRepo::find_by_slug_and_world(state.db(), &slug, request.world_id.as_ref())
        .await?
        .or_not_found("Scene", &slug)?;

    // Create new version (and cascade to derivatives)
    let (updated, version) = SceneRepo::update(state.db(), scene.id.as_ref(), &request).await?;

    Ok(Json(SceneWithVersion {
        scene: updated,
        version,
    }))
}

/// PUT /api/scenes/{id} - Update scene metadata by ID (admin)
#[instrument(skip(state, _admin, request), fields(user_id = _admin.user.id))]
async fn update_scene_metadata(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateSceneMetadataRequest>,
) -> AppResult<Json<Scene>> {
    let scene = SceneRepo::update_metadata(state.db(), &id, &request).await?;
    Ok(Json(scene))
}

/// DELETE /api/scenes/by-slug/{slug} - Disable scene by slug (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn disable_scene(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<WorldIdParam>,
) -> AppResult<StatusCode> {
    let disabled = SceneRepo::disable(state.db(), &slug, &params.world_id).await?;

    if !disabled {
        return Err(AppError::NotFound(format!("Scene '{}' not found", slug)));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/scenes/{id} - Disable scene by ID (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn disable_scene_by_id(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let disabled = SceneRepo::disable_by_id(state.db(), &id).await?;
    if !disabled {
        return Err(AppError::NotFound("Scene not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/scenes/{id}/reactivate - Reactivate a disabled scene (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn reactivate_scene(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Scene>> {
    let reactivated = SceneRepo::reactivate(state.db(), &id).await?;
    if !reactivated {
        return Err(AppError::NotFound(
            "Scene not found or already active".into(),
        ));
    }
    let scene = SceneRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("Scene", &id)?;
    Ok(Json(scene))
}

#[derive(Deserialize)]
struct BatchDisableRequest {
    slugs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BatchDisableQuery {
    world_id: String,
}

/// DELETE /api/scenes/batch - Batch disable scenes by slug within a world (admin)
#[instrument(skip(state, _admin, body), fields(user_id = _admin.user.id))]
async fn batch_disable_scenes(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<BatchDisableQuery>,
    Json(body): Json<BatchDisableRequest>,
) -> AppResult<StatusCode> {
    SceneRepo::batch_disable(state.db(), &body.slugs, &params.world_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/scenes/by-slug/{slug}/captures - Paginated captures for a scene (public)
#[instrument(skip(state))]
async fn list_scene_captures(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<PageQuery>,
) -> AppResult<Json<Paginated<CaptureWithContext>>> {
    let db = state.db();

    let scene = SceneRepo::find_active_by_slug(db, &slug)
        .await?
        .first()
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("Scene '{}' not found", slug)))?;

    let p = params.normalize();

    let filters = CaptureFilters {
        scene_id: Some(scene.id),
        status: Some(CaptureStatus::Completed),
        ..Default::default()
    };

    let (items, total) =
        CaptureRepo::list_with_context(db, &filters, Some(&p), CaptureDistinct::PerShader).await?;

    Ok(Json(Paginated::new(items, total.unwrap_or(0), &p)))
}
