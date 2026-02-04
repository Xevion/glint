use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    error::{AppError, AppResult},
    models::SceneWithCaptures,
    repo::{CaptureRepo, SceneRepo, WorldRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_scenes))
        .route("/{slug}", get(get_scene))
}

async fn list_scenes(State(state): State<AppState>) -> AppResult<Json<Vec<crate::models::Scene>>> {
    let scenes = SceneRepo::list_active(state.db()).await?;
    Ok(Json(scenes))
}

#[derive(Deserialize)]
struct SceneQuery {
    world_id: Option<String>,
}

async fn get_scene(
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
