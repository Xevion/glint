//! Main worker loop

use crate::client::ApiClient;
use crate::config::Config;
use crate::download::Downloader;
use crate::minecraft::{self, MinecraftProcess, MinecraftResult};
use anyhow::Result;
use glint_shared::{CaptureRecord, CompleteJobRequest, JobPayload, OrchestrationManifest};
use std::path::Path;
use tokio::fs;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Run the worker loop continuously
pub async fn run_loop(config: &Config) -> Result<()> {
    debug!("Worker loop started");

    loop {
        match run_once(config).await {
            Ok(()) => {
                // Job completed or no job available, continue
            }
            Err(e) => {
                error!(error = %e, "Worker error");
                // Wait before retrying on error
                tokio::time::sleep(config.poll_interval).await;
            }
        }

        // Wait before next poll
        tokio::time::sleep(config.poll_interval).await;
    }
}

/// Run development mode - directly capture shader/scenes without job queue
pub async fn run_dev_direct(
    config: &Config,
    shader_slug: &str,
    scene_slugs: &[String],
) -> Result<()> {
    let client = ApiClient::new(&config.api_url, &config.api_key);

    debug!("Fetching shader and scene data from backend");

    // Fetch shader version (use latest version for the shader)
    let shader_version = fetch_shader_version(&client, shader_slug).await?;

    // Fetch scenes by slugs (includes world data)
    let (scenes, worlds) = fetch_scenes_with_worlds(&client, scene_slugs).await?;

    if scenes.is_empty() {
        anyhow::bail!("No scenes found for slugs: {:?}", scene_slugs);
    }

    info!(
        shader = %shader_version.slug,
        version = %shader_version.version,
        scene_count = scenes.len(),
        world_count = worlds.len(),
        "Starting direct capture"
    );

    // Create a synthetic job payload
    let job = JobPayload {
        id: format!("dev-{}", chrono::Utc::now().timestamp()),
        shader: shader_version,
        scenes,
        worlds,
        profiles: Vec::new(),
        priority: 0,
    };

    // Run the capture (without backend job tracking)
    process_job(config, &client, &job).await?;

    info!("Direct capture complete");
    Ok(())
}

/// Run a single job (or return if none available)
pub async fn run_once(config: &Config) -> Result<()> {
    let client = ApiClient::new(&config.api_url, &config.api_key);

    // Try to claim a job
    let Some(job) = client.claim_job().await? else {
        debug!("No jobs available");
        return Ok(());
    };

    info!(
        job_id = %job.id,
        shader = %job.shader.slug,
        shader_version = %job.shader.version,
        scene_count = job.scenes.len(),
        "Processing job"
    );

    // Process the job with error handling
    match process_job(config, &client, &job).await {
        Ok(()) => {
            info!(job_id = %job.id, "Job completed successfully");
        }
        Err(e) => {
            error!(job_id = %job.id, error = %e, "Job failed");
            if let Err(report_err) = client.fail_job(&job.id, &e.to_string()).await {
                error!(error = %report_err, "Failed to report job failure");
            }
        }
    }

    Ok(())
}

/// Process a single job
async fn process_job(config: &Config, client: &ApiClient, job: &JobPayload) -> Result<()> {
    debug!("Setting up directories");
    // Ensure directories exist
    fs::create_dir_all(config.saves_dir()).await?;
    fs::create_dir_all(config.shaderpacks_dir()).await?;
    fs::create_dir_all(config.scenes_dir()).await?;
    fs::create_dir_all(config.captures_dir()).await?;

    // Download resources
    debug!("Downloading resources");
    let mut downloader = Downloader::new();

    for world in &job.worlds {
        debug!(world_slug = %world.slug, "Downloading world");
        downloader
            .download_world(world, &config.saves_dir())
            .await?;
    }

    debug!(shader = %job.shader.slug, version = %job.shader.version, "Downloading shader");
    downloader
        .download_shader(&job.shader, &config.shaderpacks_dir())
        .await?;

    // Write scene definitions
    debug!(scene_count = job.scenes.len(), "Writing scene definitions");
    minecraft::write_scene_definitions(&config.scenes_dir(), &job.scenes, &job.worlds).await?;

    // Remove duplicate library versions to prevent classpath conflicts
    minecraft::deduplicate_libraries(&config.libraries_dir()).await?;

    // Launch Minecraft with heartbeat
    let mc = MinecraftProcess::launch(&config.minecraft_dir, &config.java_path).await?;

    // Start heartbeat task
    let heartbeat_client = ApiClient::new(&config.api_url, &config.api_key);
    let job_id = job.id.clone();
    let heartbeat_interval = config.heartbeat_interval;

    let heartbeat_handle = tokio::spawn(async move {
        let mut interval = interval(heartbeat_interval);
        loop {
            interval.tick().await;
            if let Err(e) = heartbeat_client.heartbeat(&job_id).await {
                warn!(error = %e, "Heartbeat failed");
            }
        }
    });

    // Wait for Minecraft to complete
    debug!("Waiting for Minecraft to complete");
    let result = mc.wait().await?;

    // Stop heartbeat
    heartbeat_handle.abort();

    // Check result
    match result {
        MinecraftResult::Success => {
            debug!("Minecraft completed successfully");
        }
        MinecraftResult::Failed { code } => {
            anyhow::bail!("Minecraft exited with code {}", code);
        }
        MinecraftResult::Crashed { message } => {
            anyhow::bail!("Minecraft crashed: {}", message);
        }
    }

    // Find and parse manifest
    debug!("Looking for capture manifest");
    let manifest = find_and_parse_manifest(&config.captures_dir()).await?;

    // Upload screenshots and report completion
    debug!("Uploading screenshots");
    upload_and_complete(config, client, job, &manifest).await?;

    Ok(())
}

/// Find the latest manifest.json in captures directory
async fn find_and_parse_manifest(captures_dir: &Path) -> Result<OrchestrationManifest> {
    // Find most recent capture session directory
    let mut entries = fs::read_dir(captures_dir).await?;
    let mut latest: Option<(String, std::time::SystemTime)> = None;

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        if !metadata.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("auto_") {
            continue;
        }

        let modified = metadata.modified()?;
        if latest.as_ref().is_none_or(|(_, t)| modified > *t) {
            latest = Some((name, modified));
        }
    }

    let Some((session_name, _)) = latest else {
        anyhow::bail!("No capture session found in {:?}", captures_dir);
    };

    let manifest_path = captures_dir.join(&session_name).join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path).await?;
    let manifest: OrchestrationManifest = serde_json::from_str(&manifest_json)?;

    debug!(
        session = %session_name,
        total_sessions = manifest.orchestration.total_sessions,
        "Found manifest"
    );

    Ok(manifest)
}

/// Upload screenshots and report job completion
async fn upload_and_complete(
    config: &Config,
    client: &ApiClient,
    job: &JobPayload,
    manifest: &OrchestrationManifest,
) -> Result<()> {
    // Collect all screenshot files
    let mut files: Vec<String> = Vec::new();
    let mut captures: Vec<CaptureRecord> = Vec::new();

    for session in &manifest.sessions {
        for screenshot in &session.screenshots {
            // Build the upload path: {scene_id}/{profile}.png
            let profile = screenshot
                .shader
                .as_ref()
                .and_then(|s| s.profile.clone())
                .unwrap_or_else(|| "default".to_string());

            let upload_path = format!("{}/{}.png", session.scene_id, profile);
            files.push(upload_path.clone());

            // Parse timestamp from screenshot
            let captured_at = chrono::DateTime::parse_from_rfc3339(&screenshot.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        timestamp = %screenshot.timestamp,
                        error = %e,
                        "Failed to parse screenshot timestamp, using current time"
                    );
                    chrono::Utc::now()
                });

            // Build capture record
            captures.push(CaptureRecord {
                scene_id: session.scene_id.clone(),
                profile: screenshot.shader.as_ref().and_then(|s| s.profile.clone()),
                screenshot_path: format!(
                    "{}/{}/{}/{}",
                    job.shader.slug, job.shader.version, session.scene_id, profile
                ),
                resolution_width: screenshot.resolution.width,
                resolution_height: screenshot.resolution.height,
                captured_at,
            });
        }
    }

    // Request upload URLs
    let upload_response = client.prepare_upload(&job.id, files.clone()).await?;

    // Upload each file
    let http_client = reqwest::Client::new();

    for session in &manifest.sessions {
        let session_dir = config
            .captures_dir()
            .join(format!("auto_{}", manifest.orchestration.id))
            .join(&session.scene_id);

        for screenshot in &session.screenshots {
            let profile = screenshot
                .shader
                .as_ref()
                .and_then(|s| s.profile.clone())
                .unwrap_or_else(|| "default".to_string());

            let upload_key = format!("{}/{}.png", session.scene_id, profile);

            if let Some(upload_url) = upload_response.urls.get(&upload_key) {
                let file_path = session_dir.join(&screenshot.file);

                if file_path.exists() {
                    let file_bytes = fs::read(&file_path).await?;

                    let response = http_client
                        .put(upload_url)
                        .header("Content-Type", "image/png")
                        .body(file_bytes)
                        .send()
                        .await?;

                    if !response.status().is_success() {
                        warn!(
                            file = %screenshot.file,
                            status = %response.status(),
                            "Failed to upload screenshot"
                        );
                    } else {
                        debug!(file = %screenshot.file, "Uploaded screenshot");
                    }
                } else {
                    warn!(file = %file_path.display(), "Screenshot file not found");
                }
            }
        }
    }

    // Collect discovered profiles (deduplicated)
    let mut discovered_profiles = Vec::new();
    let mut seen_profiles = std::collections::HashSet::new();
    for session in &manifest.sessions {
        for screenshot in &session.screenshots {
            if let Some(profile) = screenshot.shader.as_ref().and_then(|sh| sh.profile.clone())
                && seen_profiles.insert(profile.clone())
            {
                discovered_profiles.push(profile);
            }
        }
    }

    // Report completion
    debug!(capture_count = captures.len(), "Reporting job completion");
    client
        .complete_job(
            &job.id,
            CompleteJobRequest {
                captures,
                discovered_profiles: if discovered_profiles.is_empty() {
                    None
                } else {
                    Some(discovered_profiles)
                },
            },
        )
        .await?;

    Ok(())
}

// =============================================================================
// Development mode helpers - fetch data directly from backend API
// =============================================================================

#[derive(Debug, serde::Deserialize)]
struct ShaderResponse {
    id: String,
    slug: String,
    name: String,
    versions: Vec<ShaderVersionResponse>,
}

#[derive(Debug, serde::Deserialize)]
struct ShaderVersionResponse {
    id: String,
    version: String,
    download_url: Option<String>,
    file_hash: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SceneDetailResponse {
    id: String,
    slug: String,
    name: String,
    world_id: String,
    definition_json: Option<String>,
    world: Option<WorldResponse>,
}

#[derive(Debug, serde::Deserialize)]
struct WorldResponse {
    id: String,
    slug: String,
    name: String,
    file_url: Option<String>,
    file_hash: Option<String>,
    size_bytes: Option<i64>,
}

async fn fetch_shader_version(
    client: &ApiClient,
    shader_slug: &str,
) -> Result<glint_shared::ShaderInfo> {
    let url = format!("{}/api/shaders/{}", client.base_url(), shader_slug);
    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch shader '{}': {}",
            shader_slug,
            response.status()
        );
    }

    let shader: ShaderResponse = response.json().await?;

    // Use the first version (latest)
    let version = shader
        .versions
        .first()
        .ok_or_else(|| anyhow::anyhow!("Shader '{}' has no versions", shader_slug))?;

    Ok(glint_shared::ShaderInfo {
        id: shader.id,
        slug: shader.slug,
        name: shader.name,
        version_id: version.id.clone(),
        version: version.version.clone(),
        download_url: version.download_url.clone(),
        file_hash: version.file_hash.clone(),
    })
}

async fn fetch_scenes_with_worlds(
    client: &ApiClient,
    scene_slugs: &[String],
) -> Result<(Vec<glint_shared::SceneInfo>, Vec<glint_shared::WorldInfo>)> {
    let mut scenes = Vec::new();
    let mut worlds = Vec::new();
    let mut seen_world_ids = std::collections::HashSet::new();

    for slug in scene_slugs {
        let url = format!("{}/api/scenes/{}", client.base_url(), slug);
        let response = reqwest::get(&url).await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch scene '{}': {}", slug, response.status());
        }

        let details: Vec<SceneDetailResponse> = response.json().await?;

        // Scenes are world-scoped, so a slug may match multiple scenes across worlds.
        // In dev mode, use the first match.
        let detail = details
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No scenes found for slug '{}'", slug))?;

        scenes.push(glint_shared::SceneInfo {
            id: detail.id.clone(),
            slug: detail.slug.clone(),
            name: detail.name.clone(),
            world_id: detail.world_id.clone(),
            definition_json: detail.definition_json.unwrap_or_else(|| "{}".to_string()),
        });

        // Add world if not already seen
        if let Some(world) = detail.world
            && seen_world_ids.insert(world.id.clone())
        {
            worlds.push(glint_shared::WorldInfo {
                id: world.id,
                slug: world.slug,
                name: world.name,
                file_url: world.file_url,
                file_hash: world.file_hash,
                size_bytes: world.size_bytes,
            });
        }
    }

    Ok((scenes, worlds))
}
