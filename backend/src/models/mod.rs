use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Downloadable world files containing scenes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct World {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub file_url: Option<String>,
    pub file_hash: Option<String>,
    pub size_bytes: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// Shader pack identity (not version-specific)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Shader {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Specific release of a shader pack
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShaderVersion {
    pub id: String,
    pub shader_id: String,
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    /// JSON array of profile names, discovered after first capture
    pub supported_profiles: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub world_id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: f64,
    pub yaw: f64,
    pub dimension: String,
    pub time_of_day_ticks: i64,
    pub weather: String,
    pub weather_intensity: f64,
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
    pub definition_json: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Capture {
    pub id: String,
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
    pub screenshot_url: Option<String>,
    pub screenshot_path: Option<String>,
    pub video_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub avg_fps: Option<f64>,
    pub min_fps: Option<f64>,
    pub max_fps: Option<f64>,
    pub frame_time_avg: Option<f64>,
    pub frame_time_p99: Option<f64>,
    pub minecraft_version: Option<String>,
    pub iris_version: Option<String>,
    pub gpu_model: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: String,
    pub shader_version_id: String,
    pub scene_ids: Option<String>,
    pub profiles: Option<String>,
    pub priority: i32,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub agent_id: Option<String>,
    pub claimed_at: Option<String>,
    pub last_heartbeat: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

// =============================================================================
// API Response Types
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ShaderWithVersions {
    #[serde(flatten)]
    pub shader: Shader,
    pub versions: Vec<ShaderVersion>,
}

#[derive(Debug, Serialize)]
pub struct ShaderWithCaptures {
    #[serde(flatten)]
    pub shader: Shader,
    pub versions: Vec<ShaderVersion>,
    pub captures: Vec<CaptureWithContext>,
}

#[derive(Debug, Serialize)]
pub struct SceneWithCaptures {
    #[serde(flatten)]
    pub scene: Scene,
    pub world: Option<World>,
    pub captures: Vec<CaptureWithContext>,
}

/// Capture with denormalized shader/version info for API responses
#[derive(Debug, Serialize, FromRow)]
pub struct CaptureWithContext {
    pub id: String,
    pub scene_id: String,
    pub shader_slug: String,
    pub shader_name: String,
    pub shader_version: String,
    pub profile: Option<String>,
    pub screenshot_path: Option<String>,
    pub screenshot_url: Option<String>,
    pub captured_at: Option<String>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
}

// =============================================================================
// Agent API Types
// =============================================================================

/// Job payload sent to agent via GET /api/agent/jobs/next
#[derive(Debug, Serialize)]
pub struct JobPayload {
    pub id: String,
    pub shader_version: ShaderVersionPayload,
    pub profiles: Vec<String>,
    pub scenes: Vec<ScenePayload>,
}

#[derive(Debug, Serialize)]
pub struct ShaderVersionPayload {
    pub id: String,
    pub shader_slug: String,
    pub shader_name: String,
    pub version: String,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScenePayload {
    pub id: String,
    pub slug: String,
    pub world: WorldPayload,
    pub definition_json: String,
}

#[derive(Debug, Serialize)]
pub struct WorldPayload {
    pub id: String,
    pub slug: String,
    pub download_url: String,
    pub file_hash: String,
}

/// Request body for POST /api/agent/jobs/{id}/prepare
#[derive(Debug, Deserialize)]
pub struct PrepareUploadRequest {
    pub files: Vec<String>,
}

/// Response for POST /api/agent/jobs/{id}/prepare
#[derive(Debug, Serialize)]
pub struct PrepareUploadResponse {
    pub uploads: std::collections::HashMap<String, String>,
}

/// Request body for POST /api/agent/jobs/{id}/complete
#[derive(Debug, Deserialize)]
pub struct CompleteJobRequest {
    pub captures: Vec<CaptureResult>,
    pub discovered_profiles: Option<Vec<String>>,
    pub metadata: Option<CaptureMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct CaptureResult {
    pub scene_id: String,
    pub profile: Option<String>,
    pub screenshot_path: String,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub captured_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CaptureMetadata {
    pub minecraft_version: Option<String>,
    pub iris_version: Option<String>,
    pub gpu_model: Option<String>,
}

/// Request body for POST /api/agent/jobs/{id}/fail
#[derive(Debug, Deserialize)]
pub struct FailJobRequest {
    pub error_message: String,
}

// =============================================================================
// Admin API Types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateShaderRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShaderVersionRequest {
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorldRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub file_url: Option<String>,
    pub file_hash: Option<String>,
    pub size_bytes: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSceneRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub world_id: String,
    pub definition_json: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub shader_version_id: String,
    pub scene_ids: Vec<String>,
    pub profiles: Option<Vec<String>>,
    pub priority: Option<i32>,
}
