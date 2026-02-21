//! Agent API types shared between the backend and the Minecraft mod.
//!
//! These types are used for API request/response serialization.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// Agent Request Types

/// A file entry in a prepare-upload request, with metadata for R2 key generation.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareUploadFile {
    pub local_path: String,
    pub scene_id: String,
    pub profile_id: Option<String>,
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
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureRecord {
    pub capture_id: String,
    pub scene_id: String,
    pub profile_id: Option<String>,
    pub image_path: String,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub captured_at: DateTime<Utc>,
}

// Orchestration Manifest Types (matches mod output)

/// The manifest.json output by the Minecraft mod
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrationManifest {
    pub orchestration: OrchestrationInfo,
    pub sessions: Vec<CaptureSessionData>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OrchestrationInfo {
    pub id: String,
    /// Run ID from the run definition (None for interactive captures)
    pub run_id: Option<String>,
    pub started_at: String,
    pub completed_at: String,
    pub total_sessions: u32,
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
    pub scene_id: String,
    pub session_dir: String,
    pub started_at: String,
    pub completed_at: String,
    pub total_captures: u32,
    pub shaders: Vec<String>,
    pub minecraft: MinecraftInfo,
    pub captures: Vec<CaptureEntry>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MinecraftInfo {
    pub version: String,
    pub iris_version: Option<String>,
    pub dimension: Option<String>,
    pub position: Option<Position>,
    pub camera: Option<Camera>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureEntry {
    pub file: String,
    pub timestamp: String,
    pub shader: Option<ShaderMetadata>,
    pub resolution: Resolution,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ShaderMetadata {
    pub filename: String,
    pub id: String,
    pub version: String,
    pub profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
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
