use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{
    error::{AppError, AppResult},
    models::{CaptureWithContext, Scene, SceneWithCaptures, World},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_scenes))
        .route("/{slug}", get(get_scene))
}

async fn list_scenes(State(state): State<AppState>) -> AppResult<Json<Vec<Scene>>> {
    let scenes = sqlx::query_as::<_, Scene>("SELECT * FROM scenes WHERE active = 1 ORDER BY name")
        .fetch_all(state.db())
        .await?;

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
    let scenes = if let Some(world_id) = &params.world_id {
        sqlx::query_as::<_, Scene>(
            "SELECT * FROM scenes WHERE slug = ?1 AND world_id = ?2 AND active = 1",
        )
        .bind(&slug)
        .bind(world_id)
        .fetch_all(state.db())
        .await?
    } else {
        sqlx::query_as::<_, Scene>("SELECT * FROM scenes WHERE slug = ?1 AND active = 1")
            .bind(&slug)
            .fetch_all(state.db())
            .await?
    };

    // Return 404 only if slug is completely unused (no scenes found at all)
    if scenes.is_empty() {
        return Err(AppError::NotFound(format!("Scene '{slug}' not found")));
    }

    // Build response with captures for each scene
    let mut results = Vec::new();
    for scene in scenes {
        // Fetch the associated world
        let world = sqlx::query_as::<_, World>("SELECT * FROM worlds WHERE id = ?1")
            .bind(&scene.world_id)
            .fetch_optional(state.db())
            .await?;

        // Fetch captures with shader/version context via JOIN
        let captures = sqlx::query_as::<_, CaptureWithContext>(
            r#"
            SELECT 
                c.id,
                c.scene_id,
                s.slug as shader_slug,
                s.name as shader_name,
                sv.version as shader_version,
                c.profile,
                c.screenshot_path,
                c.screenshot_url,
                c.captured_at,
                c.resolution_width,
                c.resolution_height
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            WHERE c.scene_id = ? AND c.status = 'completed'
            ORDER BY s.name, sv.created_at DESC
            "#,
        )
        .bind(&scene.id)
        .fetch_all(state.db())
        .await?;

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
