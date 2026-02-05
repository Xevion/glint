//! Agent API types shared between glint-backend and the Minecraft mod.
//!
//! These types are used for API request/response serialization.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Job Types

/// Job status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Claimed,
    Running,
    Completed,
    Failed,
}

/// Full job payload returned when claiming a job
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JobPayload {
    pub id: String,
    pub shader: ShaderInfo,
    pub scenes: Vec<SceneInfo>,
    pub worlds: Vec<WorldInfo>,
    pub profiles: Vec<String>,
    pub priority: i32,
}

/// Shader information for a job
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ShaderInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub version_id: String,
    pub version: String,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
}

/// Scene information for a job
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SceneInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub world_id: String,
    pub definition_json: String,
}

/// World information for a job
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorldInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub file_url: Option<String>,
    pub file_hash: Option<String>,
    pub size_bytes: Option<i64>,
}

// Agent Request Types

/// Request to prepare upload URLs
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareUploadRequest {
    pub files: Vec<String>,
}

/// Response with pre-signed upload URLs
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareUploadResponse {
    pub urls: HashMap<String, String>,
}

/// Request to complete a job
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompleteJobRequest {
    pub captures: Vec<CaptureRecord>,
    pub discovered_profiles: Option<Vec<String>>,
}

/// A single capture result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureRecord {
    pub scene_id: String,
    pub profile: Option<String>,
    pub screenshot_path: String,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub captured_at: DateTime<Utc>,
}

/// Request to fail a job
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailJobRequest {
    pub error_message: String,
}

// Orchestration Manifest Types (matches mod output)

/// The manifest.json output by the Minecraft mod
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrationManifest {
    pub orchestration: OrchestrationInfo,
    pub sessions: Vec<CaptureSessionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrationInfo {
    pub id: String,
    /// Job ID from the job definition (None for interactive captures)
    #[serde(rename = "jobId", skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "completedAt")]
    pub completed_at: String,
    #[serde(rename = "totalSessions")]
    pub total_sessions: i32,
    pub status: OrchestrationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrchestrationStatus {
    Complete,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureSessionData {
    #[serde(rename = "worldName")]
    pub world_name: String,
    #[serde(rename = "sceneId")]
    pub scene_id: String,
    #[serde(rename = "sessionDir")]
    pub session_dir: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "completedAt")]
    pub completed_at: String,
    #[serde(rename = "totalScreenshots")]
    pub total_screenshots: i32,
    #[serde(rename = "shaderPacks")]
    pub shader_packs: Vec<String>,
    pub minecraft: MinecraftInfo,
    pub screenshots: Vec<ScreenshotEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MinecraftInfo {
    pub version: String,
    #[serde(rename = "irisVersion")]
    pub iris_version: Option<String>,
    pub dimension: Option<String>,
    pub position: Option<Position>,
    pub camera: Option<Camera>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScreenshotEntry {
    pub file: String,
    pub timestamp: String,
    pub shader: Option<ShaderMetadata>,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ShaderMetadata {
    #[serde(rename = "packFile")]
    pub pack_file: String,
    pub id: String,
    pub version: String,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Resolution {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
}
