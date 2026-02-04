use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
};
use serde::Deserialize;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{
        CreateSceneRequest, Scene, SceneWithCaptures, SceneWithWorld, UpdateSceneMetadataRequest,
        UpdateSceneRequest,
    },
    repo::{CaptureRepo, SceneRepo, WorldRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_scenes_public).post(create_scene))
        .route("/all", get(list_scenes_all))
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
}

/// GET /api/scenes - List active scenes (public)
async fn list_scenes_public(State(state): State<AppState>) -> AppResult<Json<Vec<Scene>>> {
    let scenes = SceneRepo::list_active(state.db()).await?;
    Ok(Json(scenes))
}

/// GET /api/scenes/all - List all scenes including disabled (admin)
async fn list_scenes_all(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SceneWithWorld>>> {
    let scenes = SceneRepo::list_all(state.db()).await?;
    Ok(Json(scenes))
}

#[derive(Deserialize)]
struct SceneQuery {
    world_id: Option<String>,
}

/// GET /api/scenes/by-slug/{slug} - Get scene by slug with captures (public)
async fn get_scene_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<SceneQuery>,
) -> AppResult<Json<Vec<SceneWithCaptures>>> {
    // Fetch all scenes with this slug (world-scoped), optionally filtered by world_id
    let scenes = if let Some(ref world_id) = params.world_id {
        match SceneRepo::find_by_slug_and_world(state.db(), &slug, world_id).await? {
            Some(scene) => vec![scene],
            None => vec![],
        }
    } else {
        SceneRepo::find_active_by_slug(state.db(), &slug).await?
    };

    // Return 404 only if slug is completely unused (no scenes found at all)
    if scenes.is_empty() {
        return Err(AppError::NotFound(format!("Scene '{}' not found", slug)));
    }

    // Build response with captures for each scene
    let mut results = Vec::new();
    for scene in scenes {
        let world = WorldRepo::find_by_id(state.db(), &scene.world_id).await?;
        let captures = CaptureRepo::list_with_context_for_scene(state.db(), &scene.id).await?;

        let mut scene = scene;
        scene.definition_json = Some(scene.build_definition_json());

        results.push(SceneWithCaptures {
            scene,
            world,
            captures,
        });
    }

    Ok(Json(results))
}

/// GET /api/scenes/{id} - Get scene by ID (admin)
async fn get_scene_by_id(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Scene>> {
    let scene = SceneRepo::get_by_id(state.db(), &id).await?;
    Ok(Json(scene))
}

/// POST /api/scenes - Create a new scene (admin)
async fn create_scene(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<CreateSceneRequest>,
) -> AppResult<(StatusCode, Json<Scene>)> {
    // Verify world exists
    if !WorldRepo::exists_by_id(state.db(), &request.world_id).await? {
        return Err(AppError::NotFound("World not found".into()));
    }

    // Check world-scoped slug uniqueness (only active scenes)
    if SceneRepo::exists_by_slug_in_world(state.db(), &request.slug, &request.world_id).await? {
        return Err(AppError::Conflict(format!(
            "Scene with slug '{}' already exists in this world",
            request.slug
        )));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let scene = SceneRepo::create(state.db(), &id, &request).await?;

    Ok((StatusCode::CREATED, Json(scene)))
}

/// PUT /api/scenes/by-slug/{slug} - Update scene by slug (admin)
#[derive(Deserialize)]
struct WorldIdParam {
    world_id: String,
}

async fn update_scene(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateSceneRequest>,
) -> AppResult<Json<Scene>> {
    // Find scene (world-scoped, active only)
    let scene = SceneRepo::get_by_slug_and_world(state.db(), &slug, &request.world_id).await?;

    // Update scene
    let updated = SceneRepo::update(state.db(), &scene.id, &request).await?;

    // Mark captures as outdated
    CaptureRepo::mark_outdated_for_scene(state.db(), &scene.id).await?;

    Ok(Json(updated))
}

/// PUT /api/scenes/{id} - Update scene metadata by ID (admin)
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
    let scene = SceneRepo::get_by_id(state.db(), &id).await?;
    Ok(Json(scene))
}
