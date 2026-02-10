//! Agent API types shared between the backend and the Minecraft mod.
//!
//! These types are used for API request/response serialization.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// World information for a capture run
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

/// A file entry in a prepare-upload request, with metadata for R2 key generation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareUploadFile {
    pub local_path: String,
    pub scene_id: String,
    pub profile: Option<String>,
}

/// Request to prepare upload URLs
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareUploadRequest {
    pub files: Vec<PrepareUploadFile>,
}

/// A single upload target returned from prepare-upload.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UploadTarget {
    pub presigned_url: String,
    pub r2_key: String,
    pub capture_id: String,
}

/// Response with pre-signed upload URLs and capture metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareUploadResponse {
    pub uploads: HashMap<String, UploadTarget>,
}

/// A single capture result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureRecord {
    pub capture_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
    pub image_path: String,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub captured_at: DateTime<Utc>,
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
    /// Run ID from the run definition (None for interactive captures)
    #[serde(rename = "runId", skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
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
    #[serde(rename = "totalCaptures")]
    pub total_captures: i32,
    pub shaders: Vec<String>,
    pub minecraft: MinecraftInfo,
    pub captures: Vec<CaptureEntry>,
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
pub struct CaptureEntry {
    pub file: String,
    pub timestamp: String,
    pub shader: Option<ShaderMetadata>,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ShaderMetadata {
    pub filename: String,
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
    pub yaw: f64,
    pub pitch: f64,
}
