use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;

use glint_shared::{
    CompleteJobRequest, FailJobRequest, JobPayload, PrepareUploadRequest, PrepareUploadResponse,
    SceneInfo, ShaderInfo, WorldInfo,
};

use crate::{error::AppResult, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs/next", get(claim_next_job))
        .route("/jobs/{id}/heartbeat", post(heartbeat))
        .route("/jobs/{id}/prepare", post(prepare_upload))
        .route("/jobs/{id}/complete", post(complete_job))
        .route("/jobs/{id}/fail", post(fail_job))
}

/// GET /api/agent/jobs/next
/// Claims the next available job and returns the full payload
async fn claim_next_job(State(state): State<AppState>) -> AppResult<Json<Option<JobPayload>>> {
    // TODO: Extract agent_id from API key header
    let agent_id = "default-agent";

    // Find and claim the next pending job (highest priority, oldest first)
    let job = sqlx::query_as!(
        JobRow,
        r#"
        UPDATE jobs
        SET status = 'claimed',
            agent_id = $1,
            claimed_at = now(),
            last_heartbeat = now(),
            attempts = attempts + 1
        WHERE id = (
            SELECT id FROM jobs
            WHERE status = 'pending'
              AND attempts < max_attempts
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
        )
        RETURNING id, shader_version_id, scene_ids, profiles, priority
        "#,
        agent_id
    )
    .fetch_optional(state.db())
    .await?;

    let Some(job) = job else {
        return Ok(Json(None));
    };

    // Fetch shader version with shader info
    let shader_version = sqlx::query_as!(
        ShaderVersionRow,
        r#"
        SELECT
            sv.id, sv.version, sv.download_url, sv.file_hash,
            s.id as shader_id, s.slug as shader_slug, s.name as shader_name
        FROM shader_versions sv
        JOIN shaders s ON s.id = sv.shader_id
        WHERE sv.id = $1
        "#,
        job.shader_version_id
    )
    .fetch_one(state.db())
    .await?;

    // Parse scene IDs from JSON array
    let scene_ids: Vec<String> = match job.scene_ids.as_deref() {
        Some(json_str) => match serde_json::from_str(json_str) {
            Ok(ids) => ids,
            Err(e) => {
                tracing::warn!(error = %e, json = %json_str, "Failed to parse scene_ids JSON");
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    // Fetch scenes with world info
    let mut scenes = Vec::new();
    let mut worlds_map = std::collections::HashMap::new();

    for scene_id in &scene_ids {
        let scene = sqlx::query_as!(
            SceneRow,
            r#"
            SELECT
                sc.id, sc.slug, sc.name, sc.definition_json,
                sc.x, sc.y, sc.z, sc.pitch, sc.yaw,
                sc.dimension, sc.time_of_day_ticks, sc.weather, sc.weather_intensity,
                sc.moon_phase, sc.biome,
                w.id as world_id, w.slug as world_slug, w.name as world_name,
                w.file_url as world_file_url, w.file_hash as world_file_hash,
                w.size_bytes as world_size_bytes
            FROM scenes sc
            JOIN worlds w ON w.id = sc.world_id
            WHERE sc.id = $1
            "#,
            scene_id
        )
        .fetch_optional(state.db())
        .await?;

        if let Some(scene) = scene {
            let Some(ref world_file_url) = scene.world_file_url else {
                tracing::error!(scene_id = %scene.id, "Scene has no world file URL");
                continue;
            };
            let Some(ref world_file_hash) = scene.world_file_hash else {
                tracing::error!(scene_id = %scene.id, "Scene has no world file hash");
                continue;
            };

            // Add world to map if not already present
            worlds_map
                .entry(scene.world_id.clone())
                .or_insert(WorldInfo {
                    id: scene.world_id.clone(),
                    slug: scene.world_slug.clone(),
                    name: scene.world_name.clone(),
                    file_url: Some(world_file_url.clone()),
                    file_hash: Some(world_file_hash.clone()),
                    size_bytes: scene.world_size_bytes,
                });

            let definition_json = build_scene_definition_json(&scene);
            scenes.push(SceneInfo {
                id: scene.id,
                slug: scene.slug,
                name: scene.name,
                world_id: scene.world_id,
                definition_json,
            });
        }
    }

    // Parse profiles from JSON array
    let profiles: Vec<String> = match job.profiles.as_deref() {
        Some(json_str) => match serde_json::from_str(json_str) {
            Ok(profiles) => profiles,
            Err(e) => {
                tracing::warn!(error = %e, json = %json_str, "Failed to parse profiles JSON");
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    let payload = JobPayload {
        id: job.id,
        shader: ShaderInfo {
            id: shader_version.shader_id,
            slug: shader_version.shader_slug,
            name: shader_version.shader_name,
            version_id: shader_version.id.clone(),
            version: shader_version.version,
            download_url: shader_version.download_url,
            file_hash: shader_version.file_hash,
        },
        scenes,
        worlds: worlds_map.into_values().collect(),
        profiles,
        priority: job.priority,
    };

    Ok(Json(Some(payload)))
}

/// POST /api/agent/jobs/{id}/heartbeat
/// Updates last_heartbeat timestamp to keep job alive
async fn heartbeat(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET last_heartbeat = now(),
            status = CASE WHEN status = 'claimed' THEN 'running' ELSE status END
        WHERE id = $1 AND status IN ('claimed', 'running')
        "#,
        job_id
    )
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

/// POST /api/agent/jobs/{id}/prepare
/// Returns pre-signed upload URLs for the given file paths
async fn prepare_upload(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<PrepareUploadRequest>,
) -> AppResult<Json<PrepareUploadResponse>> {
    // Verify job exists and is running
    let job = sqlx::query_scalar!(
        "SELECT id FROM jobs WHERE id = $1 AND status = 'running'",
        job_id
    )
    .fetch_optional(state.db())
    .await?;

    if job.is_none() {
        return Err(crate::error::AppError::NotFound(
            "Job not found or not running".into(),
        ));
    }

    let mut urls = std::collections::HashMap::new();

    // Check if R2/S3 is configured
    let Some(s3) = state.s3() else {
        // R2 not configured - return placeholder URLs for development
        for file in request.files {
            let url = format!("https://r2.example.com/captures/{}/{}", job_id, file);
            urls.insert(file, url);
        }
        return Ok(Json(PrepareUploadResponse { urls }));
    };

    let r2_config = &state.config().r2;
    let bucket = r2_config.bucket.as_deref().unwrap_or("glint");

    // Generate pre-signed URLs for each file
    for file in request.files {
        // Structure: captures/{job_id}/{file}
        let key = format!("captures/{}/{}", job_id, file);

        let presigned = s3
            .put_object()
            .bucket(bucket)
            .key(&key)
            .content_type("image/png")
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::builder()
                    .expires_in(std::time::Duration::from_secs(3600)) // 1 hour
                    .build()
                    .expect("valid presigning config"),
            )
            .await
            .map_err(|e| {
                crate::error::AppError::Internal(anyhow::anyhow!(
                    "Failed to generate presigned URL: {}",
                    e
                ))
            })?;

        urls.insert(file, presigned.uri().to_string());
    }

    Ok(Json(PrepareUploadResponse { urls }))
}

/// POST /api/agent/jobs/{id}/complete
/// Marks job as completed and records capture results
async fn complete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<CompleteJobRequest>,
) -> AppResult<StatusCode> {
    // Get the job to find shader_version_id
    let job = sqlx::query_as!(
        JobVersionRow,
        "SELECT shader_version_id FROM jobs WHERE id = $1 AND status = 'running'",
        job_id
    )
    .fetch_optional(state.db())
    .await?;

    let Some(job) = job else {
        return Err(crate::error::AppError::NotFound(
            "Job not found or not running".into(),
        ));
    };

    // Insert captures
    for capture in &request.captures {
        let capture_id = Uuid::new_v4().to_string();
        let captured_at = capture.captured_at;
        sqlx::query!(
            r#"
            INSERT INTO captures (
                id, shader_version_id, scene_id, profile, screenshot_path,
                resolution_width, resolution_height, outdated, status, created_at, updated_at, captured_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, 'completed', $8, $8, $8)
            ON CONFLICT (shader_version_id, scene_id, profile)
            DO UPDATE SET
                screenshot_path = excluded.screenshot_path,
                resolution_width = excluded.resolution_width,
                resolution_height = excluded.resolution_height,
                outdated = FALSE,
                status = 'completed',
                updated_at = excluded.updated_at,
                captured_at = excluded.captured_at
            "#,
            capture_id,
            job.shader_version_id,
            capture.scene_id,
            capture.profile,
            capture.screenshot_path,
            capture.resolution_width,
            capture.resolution_height,
            captured_at
        )
        .execute(state.db())
        .await?;
    }

    // Update discovered profiles on shader version if provided
    if let Some(profiles) = &request.discovered_profiles {
        let profiles_json = serde_json::to_string(profiles).unwrap_or_else(|_| "[]".to_string());
        sqlx::query!(
            "UPDATE shader_versions SET supported_profiles = $1 WHERE id = $2",
            profiles_json,
            job.shader_version_id
        )
        .execute(state.db())
        .await?;
    }

    // Mark job as completed
    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'completed', completed_at = now()
        WHERE id = $1
        "#,
        job_id
    )
    .execute(state.db())
    .await?;

    Ok(StatusCode::OK)
}

/// POST /api/agent/jobs/{id}/fail
/// Marks job as failed with error message
async fn fail_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<FailJobRequest>,
) -> AppResult<StatusCode> {
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'failed',
            error_message = $1,
            completed_at = now()
        WHERE id = $2 AND status IN ('claimed', 'running')
        "#,
        request.error_message,
        job_id
    )
    .execute(state.db())
    .await?;

    if result.rows_affected() == 0 {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

struct JobRow {
    id: String,
    shader_version_id: String,
    scene_ids: Option<String>,
    profiles: Option<String>,
    priority: i32,
}

struct JobVersionRow {
    shader_version_id: String,
}

struct ShaderVersionRow {
    id: String,
    version: String,
    download_url: Option<String>,
    file_hash: Option<String>,
    shader_id: String,
    shader_slug: String,
    shader_name: String,
}

struct SceneRow {
    id: String,
    slug: String,
    name: String,
    definition_json: Option<String>,
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
    world_id: String,
    world_slug: String,
    world_name: String,
    world_file_url: Option<String>,
    world_file_hash: Option<String>,
    world_size_bytes: Option<i64>,
}

/// Builds definition JSON from SceneRow columns, matching the mod's Scene data class format.
fn build_scene_definition_json(scene: &SceneRow) -> String {
    if let Some(ref json) = scene.definition_json
        && json != "{}"
    {
        return json.clone();
    }

    let weather = scene.weather.to_uppercase();

    let mut json = serde_json::json!({
        "id": scene.slug,
        "name": scene.name,
        "position": {
            "x": scene.x,
            "y": scene.y,
            "z": scene.z
        },
        "camera": {
            "yaw": scene.yaw,
            "pitch": scene.pitch
        },
        "timeOfDay": scene.time_of_day_ticks,
        "dimension": scene.dimension,
        "weather": weather,
        "weatherIntensity": scene.weather_intensity
    });

    if let Some(ref biome) = scene.biome {
        json["biome"] = serde_json::Value::String(biome.clone());
    }

    if let Some(moon_phase) = scene.moon_phase {
        json["moonPhase"] = serde_json::Value::Number(moon_phase.into());
    }

    json.to_string()
}
