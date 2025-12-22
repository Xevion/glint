use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, delete, put},
};
use crate::db::UtcDateTime;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{
        CreateJobRequest, CreateSceneRequest, CreateShaderRequest, CreateShaderVersionRequest,
        CreateWorldRequest, Job, Scene, Shader, ShaderVersion, World,
    },
    state::AppState,
};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shaders", post(create_shader))
        .route("/shaders/{id}/versions", post(create_shader_version))
        .route("/worlds", post(create_world))
        .route("/scenes", post(create_scene))
        .route("/jobs", get(list_jobs).post(create_job))
        .route("/jobs/{id}", delete(delete_job))
        .route("/jobs/{id}/cancel", put(cancel_job))
        .route("/jobs/{id}/retry", put(retry_job))
        .route("/jobs/{id}/release", put(release_job))
}

/// POST /api/admin/shaders
async fn create_shader(
    State(state): State<AppState>,
    Json(request): Json<CreateShaderRequest>,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let id = Uuid::new_v4().to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO shaders (id, name, slug, description, modrinth_id, curseforge_id, website_url, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now', 'utc'), datetime('now', 'utc'))
        "#,
    )
    .bind(&id)
    .bind(&request.name)
    .bind(&request.slug)
    .bind(&request.description)
    .bind(&request.modrinth_id)
    .bind(&request.curseforge_id)
    .bind(&request.website_url)
    .execute(state.db())
    .await;

    if let Err(sqlx::Error::Database(db_err)) = &result
        && let Some(code) = db_err.code()
        && code == "2067"
    {
        return Err(crate::error::AppError::Conflict(format!(
            "Shader with slug '{}' already exists",
            request.slug
        )));
    }
    result?;

    let shader = sqlx::query_as::<_, Shader>("SELECT * FROM shaders WHERE id = ?1")
        .bind(&id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(shader)))
}

/// POST /api/admin/shaders/{id}/versions
async fn create_shader_version(
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
    Json(request): Json<CreateShaderVersionRequest>,
) -> AppResult<(StatusCode, Json<ShaderVersion>)> {
    // Verify shader exists
    let exists = sqlx::query_scalar::<_, i32>("SELECT 1 FROM shaders WHERE id = ?1")
        .bind(&shader_id)
        .fetch_optional(state.db())
        .await?;

    if exists.is_none() {
        return Err(crate::error::AppError::NotFound("Shader not found".into()));
    }

    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO shader_versions (id, shader_id, version, modrinth_version_id, download_url, file_hash, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now', 'utc'))
        "#,
    )
    .bind(&id)
    .bind(&shader_id)
    .bind(&request.version)
    .bind(&request.modrinth_version_id)
    .bind(&request.download_url)
    .bind(&request.file_hash)
    .execute(state.db())
    .await?;

    let version = sqlx::query_as::<_, ShaderVersion>("SELECT * FROM shader_versions WHERE id = ?1")
        .bind(&id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(version)))
}

/// POST /api/admin/worlds
async fn create_world(
    State(state): State<AppState>,
    Json(request): Json<CreateWorldRequest>,
) -> AppResult<(StatusCode, Json<World>)> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO worlds (id, name, slug, description, minecraft_version, file_url, file_hash, size_bytes, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now', 'utc'), datetime('now', 'utc'))
        "#,
    )
    .bind(&id)
    .bind(&request.name)
    .bind(&request.slug)
    .bind(&request.description)
    .bind(&request.minecraft_version)
    .bind(&request.file_url)
    .bind(&request.file_hash)
    .bind(request.size_bytes)
    .execute(state.db())
    .await?;

    let world = sqlx::query_as::<_, World>("SELECT * FROM worlds WHERE id = ?1")
        .bind(&id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(world)))
}

/// POST /api/admin/scenes
async fn create_scene(
    State(state): State<AppState>,
    Json(request): Json<CreateSceneRequest>,
) -> AppResult<(StatusCode, Json<Scene>)> {
    // Verify world exists
    let exists = sqlx::query_scalar::<_, i32>("SELECT 1 FROM worlds WHERE id = ?1")
        .bind(&request.world_id)
        .fetch_optional(state.db())
        .await?;

    if exists.is_none() {
        return Err(crate::error::AppError::NotFound("World not found".into()));
    }

    let id = Uuid::new_v4().to_string();

    // Parse definition_json to extract scene properties
    let scene_props = parse_scene_definition(&request.definition_json);

    let tags_json = request
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "[]".to_string()));

    sqlx::query(
        r#"
        INSERT INTO scenes (
            id, name, slug, description, world_id, x, y, z, pitch, yaw,
            dimension, time_of_day_ticks, weather, weather_intensity, moon_phase, biome,
            definition_json, tags, created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, datetime('now', 'utc'))
        "#,
    )
    .bind(&id)
    .bind(&request.name)
    .bind(&request.slug)
    .bind(&request.description)
    .bind(&request.world_id)
    .bind(scene_props.x)
    .bind(scene_props.y)
    .bind(scene_props.z)
    .bind(scene_props.pitch)
    .bind(scene_props.yaw)
    .bind(scene_props.dimension)
    .bind(scene_props.time_of_day_ticks)
    .bind(scene_props.weather)
    .bind(scene_props.weather_intensity)
    .bind(scene_props.moon_phase)
    .bind(scene_props.biome)
    .bind(&request.definition_json)
    .bind(&tags_json)
    .execute(state.db())
    .await?;

    let scene = sqlx::query_as::<_, Scene>("SELECT * FROM scenes WHERE id = ?1")
        .bind(&id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(scene)))
}

/// GET /api/admin/jobs
async fn list_jobs(State(state): State<AppState>) -> AppResult<Json<Vec<JobWithDetails>>> {
    let jobs = sqlx::query_as::<_, JobWithDetails>(
        r#"
        SELECT 
            j.id, j.shader_version_id, j.scene_ids, j.profiles, j.priority,
            j.status, j.attempts, j.max_attempts, j.agent_id, j.claimed_at,
            j.last_heartbeat, j.started_at, j.completed_at, j.error_message,
            j.created_at,
            s.name as shader_name,
            s.slug as shader_slug,
            sv.version as shader_version,
            COALESCE(json_array_length(j.scene_ids), 0) as scene_count
        FROM jobs j
        JOIN shader_versions sv ON sv.id = j.shader_version_id
        JOIN shaders s ON s.id = sv.shader_id
        ORDER BY j.created_at DESC
        "#,
    )
    .fetch_all(state.db())
    .await?;

    Ok(Json(jobs))
}

/// POST /api/admin/jobs
async fn create_job(
    State(state): State<AppState>,
    Json(request): Json<CreateJobRequest>,
) -> AppResult<(StatusCode, Json<Job>)> {
    // Verify shader version exists
    let exists = sqlx::query_scalar::<_, i32>("SELECT 1 FROM shader_versions WHERE id = ?1")
        .bind(&request.shader_version_id)
        .fetch_optional(state.db())
        .await?;

    if exists.is_none() {
        return Err(crate::error::AppError::NotFound(
            "Shader version not found".into(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let scene_ids_json =
        serde_json::to_string(&request.scene_ids).unwrap_or_else(|_| "[]".to_string());
    let profiles_json = request
        .profiles
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_else(|_| "[]".to_string()));
    let priority = request.priority.unwrap_or(0);

    sqlx::query(
        r#"
        INSERT INTO jobs (id, shader_version_id, scene_ids, profiles, priority, status, attempts, max_attempts, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, 'pending', 0, 3, datetime('now', 'utc'))
        "#,
    )
    .bind(&id)
    .bind(&request.shader_version_id)
    .bind(&scene_ids_json)
    .bind(&profiles_json)
    .bind(priority)
    .execute(state.db())
    .await?;

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?1")
        .bind(&id)
        .fetch_one(state.db())
        .await?;

    Ok((StatusCode::CREATED, Json(job)))
}

/// Helper to parse scene definition JSON into structured properties
fn parse_scene_definition(definition_json: &str) -> SceneProperties {
    let definition: serde_json::Value =
        serde_json::from_str(definition_json).unwrap_or(serde_json::json!({}));

    SceneProperties {
        x: definition
            .get("position")
            .and_then(|p| p.get("x"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        y: definition
            .get("position")
            .and_then(|p| p.get("y"))
            .and_then(|v| v.as_f64())
            .unwrap_or(64.0),
        z: definition
            .get("position")
            .and_then(|p| p.get("z"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        pitch: definition
            .get("camera")
            .and_then(|c| c.get("pitch"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        yaw: definition
            .get("camera")
            .and_then(|c| c.get("yaw"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        dimension: definition
            .get("dimension")
            .and_then(|v| v.as_str())
            .unwrap_or("minecraft:overworld")
            .to_string(),
        time_of_day_ticks: definition
            .get("timeOfDay")
            .and_then(|v| v.as_i64())
            .unwrap_or(6000),
        weather: definition
            .get("weather")
            .and_then(|v| v.as_str())
            .unwrap_or("CLEAR")
            .to_string(),
        weather_intensity: definition
            .get("weatherIntensity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        moon_phase: definition
            .get("moonPhase")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
        biome: definition
            .get("biome")
            .and_then(|v| v.as_str())
            .map(String::from),
    }
}

struct SceneProperties {
    x: f64,
    y: f64,
    z: f64,
    pitch: f64,
    yaw: f64,
    dimension: String,
    time_of_day_ticks: i64,
    weather: String,
    weather_intensity: f64,
    moon_phase: Option<i32>,
    biome: Option<String>,
}

#[derive(sqlx::FromRow, Serialize)]
struct JobWithDetails {
    id: String,
    shader_version_id: String,
    scene_ids: Option<String>,
    profiles: Option<String>,
    priority: i32,
    status: String,
    attempts: i32,
    max_attempts: i32,
    agent_id: Option<String>,
    claimed_at: Option<UtcDateTime>,
    last_heartbeat: Option<UtcDateTime>,
    started_at: Option<UtcDateTime>,
    completed_at: Option<UtcDateTime>,
    error_message: Option<String>,
    created_at: UtcDateTime,
    shader_name: String,
    shader_slug: String,
    shader_version: String,
    scene_count: i32,
}

/// DELETE /api/admin/jobs/{id}
async fn delete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM jobs WHERE id = ?1")
        .bind(&job_id)
        .execute(state.db())
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::NotFound("Job not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/admin/jobs/{id}/cancel
async fn cancel_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    // Only cancel pending or claimed jobs
    let result = sqlx::query(
        r#"
        UPDATE jobs 
        SET status = 'failed', 
            error_message = 'Cancelled by admin',
            completed_at = datetime('now', 'utc')
        WHERE id = ?1 AND status IN ('pending', 'claimed')
        "#,
    )
    .bind(&job_id)
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::Conflict(
            "Job not found or cannot be cancelled (only pending/claimed jobs can be cancelled)"
                .into(),
        ));
    }

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?1")
        .bind(&job_id)
        .fetch_one(state.db())
        .await?;

    Ok(Json(job))
}

/// PUT /api/admin/jobs/{id}/retry
async fn retry_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    // Only retry failed jobs
    let result = sqlx::query(
        r#"
        UPDATE jobs 
        SET status = 'pending',
            attempts = 0,
            error_message = NULL,
            agent_id = NULL,
            claimed_at = NULL,
            last_heartbeat = NULL,
            started_at = NULL,
            completed_at = NULL
        WHERE id = ?1 AND status = 'failed'
        "#,
    )
    .bind(&job_id)
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::Conflict(
            "Job not found or cannot be retried (only failed jobs can be retried)".into(),
        ));
    }

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?1")
        .bind(&job_id)
        .fetch_one(state.db())
        .await?;

    Ok(Json(job))
}

/// PUT /api/admin/jobs/{id}/release
async fn release_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<Job>> {
    // Release claimed or running jobs
    let result = sqlx::query(
        r#"
        UPDATE jobs 
        SET status = 'pending',
            agent_id = NULL,
            claimed_at = NULL,
            last_heartbeat = NULL
        WHERE id = ?1 AND status IN ('claimed', 'running')
        "#,
    )
    .bind(&job_id)
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::error::AppError::Conflict(
            "Job not found or cannot be released (only claimed/running jobs can be released)"
                .into(),
        ));
    }

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?1")
        .bind(&job_id)
        .fetch_one(state.db())
        .await?;

    Ok(Json(job))
}
