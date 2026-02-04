use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    error::{AppError, AppResult},
    models::{CaptureWithContext, Shader, ShaderVersion, ShaderWithCaptures},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shaders))
        .route("/{slug}", get(get_shader))
}

async fn list_shaders(State(state): State<AppState>) -> AppResult<Json<Vec<Shader>>> {
    let shaders = sqlx::query_as!(Shader, "SELECT * FROM shaders ORDER BY name")
        .fetch_all(state.db())
        .await?;

    Ok(Json(shaders))
}

async fn get_shader(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<ShaderWithCaptures>> {
    let shader = sqlx::query_as!(Shader, "SELECT * FROM shaders WHERE slug = $1", slug)
        .fetch_optional(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Shader '{slug}' not found")))?;

    let versions = sqlx::query_as!(
        ShaderVersion,
        "SELECT * FROM shader_versions WHERE shader_id = $1",
        shader.id
    )
    .fetch_all(state.db())
    .await?;

    // Fetch captures with shader/version context via JOIN
    let captures = sqlx::query_as!(
        CaptureWithContext,
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
        WHERE s.id = $1 AND c.status = 'completed'
        ORDER BY sv.created_at DESC
        "#,
        shader.id
    )
    .fetch_all(state.db())
    .await?;

    Ok(Json(ShaderWithCaptures {
        shader,
        versions,
        captures,
    }))
}
