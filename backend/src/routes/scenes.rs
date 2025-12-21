use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

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
    let scenes = sqlx::query_as::<_, Scene>("SELECT * FROM scenes ORDER BY name")
        .fetch_all(state.db())
        .await?;

    Ok(Json(scenes))
}

async fn get_scene(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<SceneWithCaptures>> {
    let scene = sqlx::query_as::<_, Scene>("SELECT * FROM scenes WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scene '{slug}' not found")))?;

    // Fetch the associated world
    let world = sqlx::query_as::<_, World>("SELECT * FROM worlds WHERE id = ?")
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

    Ok(Json(SceneWithCaptures {
        scene,
        world,
        captures,
    }))
}
