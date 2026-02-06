use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use nanoid::nanoid;
use rand::seq::SliceRandom;
use tracing::{info, warn};

use crate::models::CreateJobRequest;
use crate::models::agent::{
    CompleteJobRequest, FailJobRequest, ForceJobRequest, JobPayload, PrepareUploadRequest,
    PrepareUploadResponse, SceneInfo, ShaderInfo, UploadTarget, WorldInfo,
};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::repo::{CaptureRepo, JobRepo, SceneRepo, ShaderRepo, ShaderVersionRepo, WorldRepo};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs/next", get(claim_next_job))
        .route("/jobs/force", post(force_job))
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

    let job = JobRepo::claim_next(state.db(), agent_id).await?;

    let Some(job) = job else {
        return Ok(Json(None));
    };

    // Fetch shader version with shader info
    let shader_version = fetch_shader_version_with_info(state.db(), &job.shader_version_id).await?;

    // Parse scene IDs from JSON array
    let scene_ids: Vec<String> = match job.scene_ids.as_deref() {
        Some(json_str) => match serde_json::from_str(json_str) {
            Ok(ids) => ids,
            Err(e) => {
                warn!(error = %e, json = %json_str, "Failed to parse scene_ids JSON");
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    // Fetch scenes with world info
    let mut scenes = Vec::new();
    let mut worlds_map = std::collections::HashMap::new();

    for scene_id in &scene_ids {
        let scene = fetch_scene_with_world(state.db(), scene_id).await?;

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
                warn!(error = %e, json = %json_str, "Failed to parse profiles JSON");
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

/// POST /api/agent/jobs/force
/// Force-creates a job from available resources using selector syntax.
/// Used by `just orchestrate --force` for testing the capture pipeline.
async fn force_job(
    State(state): State<AppState>,
    Json(request): Json<ForceJobRequest>,
) -> AppResult<Json<JobPayload>> {
    let db = state.db();
    let scene_selector = request.scenes.as_deref().unwrap_or("+");
    let shader_selector = request.shaders.as_deref().unwrap_or("+");

    // Resolve scenes
    let all_scenes = SceneRepo::list_active(db).await?;
    if all_scenes.is_empty() {
        return Err(AppError::BadRequest("No active scenes in database".into()));
    }
    let selected_scenes = apply_selector(&all_scenes, scene_selector, |s| &s.slug)?;

    // Resolve shaders and their latest versions
    let all_shaders = ShaderRepo::list(db).await?;
    if all_shaders.is_empty() {
        return Err(AppError::BadRequest("No shaders in database".into()));
    }
    let selected_shaders = apply_selector(&all_shaders, shader_selector, |s| &s.slug)?;

    // Pair each shader with its latest version, skipping those without versions
    let mut shader_versions = Vec::new();
    for &shader in &selected_shaders {
        match ShaderVersionRepo::list_by_shader(db, &shader.id)
            .await?
            .into_iter()
            .next()
        {
            Some(version) => shader_versions.push((shader, version)),
            None => warn!(shader = %shader.slug, "Shader has no versions, skipping"),
        }
    }
    if shader_versions.is_empty() {
        return Err(AppError::BadRequest(
            "No selected shaders have versions".into(),
        ));
    }

    // Create a job for each shader (all share the same scenes)
    let scene_ids: Vec<String> = selected_scenes.iter().map(|s| s.id.clone()).collect();
    let mut job_ids = Vec::with_capacity(shader_versions.len());

    for (_, version) in &shader_versions {
        let job_id = nanoid!();
        let create_req = CreateJobRequest {
            shader_version_id: version.id.clone(),
            scene_ids: scene_ids.clone(),
            profiles: None,
            priority: Some(100),
        };
        JobRepo::create(db, &job_id, &create_req).await?;
        job_ids.push(job_id);
    }

    // Claim the first job
    let first_job_id = &job_ids[0];
    let (first_shader, first_version) = &shader_versions[0];
    let agent_id = "force-orchestrate";
    let claimed = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'claimed', agent_id = $1, claimed_at = now(), last_heartbeat = now(), attempts = attempts + 1
        WHERE id = $2
        "#,
        agent_id,
        first_job_id
    )
    .execute(db)
    .await
    .context("failed to claim forced job")?;

    if claimed.rows_affected() == 0 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Failed to claim just-created job"
        )));
    }

    // Build payload for the first job
    let shader_info = ShaderInfo {
        id: first_shader.id.clone(),
        slug: first_shader.slug.clone(),
        name: first_shader.name.clone(),
        version_id: first_version.id.clone(),
        version: first_version.version.clone(),
        download_url: first_version.download_url.clone(),
        file_hash: first_version.file_hash.clone(),
    };

    let mut scenes = Vec::new();
    let mut worlds_map = std::collections::HashMap::new();

    for scene in &selected_scenes {
        let world = WorldRepo::find_by_id(db, &scene.world_id)
            .await?
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "World '{}' not found for scene '{}'",
                    scene.world_id,
                    scene.slug
                ))
            })?;

        worlds_map.entry(world.id.clone()).or_insert(WorldInfo {
            id: world.id.clone(),
            slug: world.slug.clone(),
            name: world.name.clone(),
            file_url: world.file_url.clone(),
            file_hash: world.file_hash.clone(),
            size_bytes: world.size_bytes,
        });

        scenes.push(SceneInfo {
            id: scene.id.clone(),
            slug: scene.slug.clone(),
            name: scene.name.clone(),
            world_id: scene.world_id.clone(),
            definition_json: scene.build_definition_json(),
        });
    }

    let payload = JobPayload {
        id: first_job_id.clone(),
        shader: shader_info,
        scenes,
        worlds: worlds_map.into_values().collect(),
        profiles: Vec::new(),
        priority: 100,
    };

    info!(
        first_job = %first_job_id,
        job_count = job_ids.len(),
        shader_count = shader_versions.len(),
        scene_count = selected_scenes.len(),
        "Force-created jobs"
    );

    Ok(Json(payload))
}

/// Parses a selector string and applies it to a list of items.
///
/// Syntax: `"+"` (1 random), `"!"` (all), `"3+"` (3 random), `"slug"` (by slug)
fn apply_selector<'a, T>(
    items: &'a [T],
    selector: &str,
    get_slug: impl Fn(&T) -> &str,
) -> AppResult<Vec<&'a T>> {
    let selector = selector.trim();

    // All
    if selector == "*" || selector == "!" {
        return Ok(items.iter().collect());
    }

    // Random N: "+" or "N+"
    if selector == "+" || selector.ends_with('+') {
        let count = if selector == "+" {
            1
        } else {
            selector
                .trim_end_matches('+')
                .parse::<usize>()
                .map_err(|_| AppError::BadRequest(format!("Invalid selector: '{selector}'")))?
        };

        let count = count.min(items.len());
        let mut indices: Vec<usize> = (0..items.len()).collect();
        indices.shuffle(&mut rand::rng());
        indices.truncate(count);
        return Ok(indices.into_iter().map(|i| &items[i]).collect());
    }

    // Specific slug
    let found = items.iter().find(|item| get_slug(item) == selector);
    match found {
        Some(item) => Ok(vec![item]),
        None => Err(AppError::NotFound(format!(
            "No item with slug '{selector}'"
        ))),
    }
}

/// POST /api/agent/jobs/{id}/heartbeat
/// Updates last_heartbeat timestamp to keep job alive
async fn heartbeat(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<StatusCode> {
    let updated = JobRepo::heartbeat(state.db(), &job_id).await?;

    if !updated {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

/// POST /api/agent/jobs/{id}/prepare
/// Returns pre-signed upload URLs with semantic R2 keys for each file.
async fn prepare_upload(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<PrepareUploadRequest>,
) -> AppResult<Json<PrepareUploadResponse>> {
    let db = state.db();

    // Get shader_version_id for active job
    let shader_version_id = JobRepo::get_active_shader_version_id(db, &job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found or not active".into()))?;

    // Resolve the shader slug for R2 key generation
    let shader_slug = ShaderVersionRepo::get_shader_slug(db, &shader_version_id)
        .await?
        .ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "Shader version '{}' has no parent shader",
                shader_version_id
            ))
        })?;

    let mut uploads = std::collections::HashMap::new();

    // Check if R2/S3 is configured
    let Some(s3) = state.s3() else {
        // R2 not configured — return placeholder URLs for development
        for file in request.files {
            let capture_id = nanoid!();
            let r2_key = format!(
                "captures/{}/{}/{}.png",
                shader_slug, file.scene_id, capture_id
            );
            uploads.insert(
                file.local_path,
                UploadTarget {
                    presigned_url: format!("https://r2.example.com/{}", r2_key),
                    r2_key,
                    capture_id,
                },
            );
        }
        return Ok(Json(PrepareUploadResponse { uploads }));
    };

    let r2_config = &state.config().r2;
    let bucket = r2_config.bucket.as_deref().unwrap_or("glint");

    for file in request.files {
        let capture_id = nanoid!();
        let r2_key = format!(
            "captures/{}/{}/{}.png",
            shader_slug, file.scene_id, capture_id
        );

        let presigned = s3
            .put_object()
            .bucket(bucket)
            .key(&r2_key)
            .content_type("image/png")
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::builder()
                    .expires_in(std::time::Duration::from_secs(3600))
                    .build()
                    .expect("valid presigning config"),
            )
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to generate presigned URL: {}", e))
            })?;

        uploads.insert(
            file.local_path,
            UploadTarget {
                presigned_url: presigned.uri().to_string(),
                r2_key,
                capture_id,
            },
        );
    }

    Ok(Json(PrepareUploadResponse { uploads }))
}

/// POST /api/agent/jobs/{id}/complete
/// Marks job as completed and records capture results.
/// Capture IDs come from the prepare-upload step.
async fn complete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<CompleteJobRequest>,
) -> AppResult<StatusCode> {
    let db = state.db();

    // Get shader_version_id for active job (claimed or running)
    let shader_version_id = JobRepo::get_active_shader_version_id(db, &job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found or not active".into()))?;

    // Resolve shader slug to reconstruct R2 keys
    let shader_slug = ShaderVersionRepo::get_shader_slug(db, &shader_version_id)
        .await?
        .ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "Shader version '{}' has no parent shader",
                shader_version_id
            ))
        })?;

    let r2_config = &state.config().r2;
    for capture in &request.captures {
        let r2_key = format!(
            "captures/{}/{}/{}.png",
            shader_slug, capture.scene_id, capture.capture_id
        );
        let screenshot_url = r2_config.public_url_for_key(&r2_key);
        CaptureRepo::upsert(
            db,
            &capture.capture_id,
            &shader_version_id,
            &capture.scene_id,
            capture.profile.as_deref(),
            Some(&capture.screenshot_path),
            Some(&screenshot_url),
            Some(capture.resolution_width),
            Some(capture.resolution_height),
            Some(capture.captured_at),
        )
        .await?;
    }

    // Update discovered profiles on shader version if provided
    if let Some(profiles) = &request.discovered_profiles {
        let profiles_json = serde_json::to_string(profiles).unwrap_or_else(|_| "[]".to_string());
        ShaderVersionRepo::update_supported_profiles(db, &shader_version_id, &profiles_json)
            .await?;
    }

    // Mark job as completed
    JobRepo::complete(db, &job_id).await?;

    Ok(StatusCode::OK)
}

/// POST /api/agent/jobs/{id}/fail
/// Marks job as failed with error message
async fn fail_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<FailJobRequest>,
) -> AppResult<StatusCode> {
    let updated = JobRepo::fail(state.db(), &job_id, &request.error_message).await?;

    if !updated {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

// Helper structs for complex queries

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

async fn fetch_shader_version_with_info(
    db: &DbPool,
    shader_version_id: &str,
) -> AppResult<ShaderVersionRow> {
    sqlx::query_as!(
        ShaderVersionRow,
        r#"
        SELECT
            sv.id, sv.version, sv.download_url, sv.file_hash,
            s.id as shader_id, s.slug as shader_slug, s.name as shader_name
        FROM shader_versions sv
        JOIN shaders s ON s.id = sv.shader_id
        WHERE sv.id = $1
        "#,
        shader_version_id
    )
    .fetch_one(db)
    .await
    .context(format!(
        "failed to fetch shader version '{}'",
        shader_version_id
    ))
    .map_err(Into::into)
}

async fn fetch_scene_with_world(db: &DbPool, scene_id: &str) -> AppResult<Option<SceneRow>> {
    sqlx::query_as!(
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
    .fetch_optional(db)
    .await
    .context(format!("failed to fetch scene with world '{}'", scene_id))
    .map_err(Into::into)
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
