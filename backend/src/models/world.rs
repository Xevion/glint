use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

use super::scene::Scene;

/// Downloadable world files containing scenes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct World {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// A specific revision of a World's save file
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct WorldVersion {
    pub id: String,
    pub world_id: String,
    pub file_url: Option<String>,
    pub file_hash: Option<String>,
    pub size_bytes: Option<i64>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

/// Tracks pending world uploads (presigned URL workflow).
/// World creation uploads set slug/name/minecraft_version.
/// Version uploads set world_id instead.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PendingUpload {
    pub upload_id: String,
    pub world_id: Option<String>,
    pub slug: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub minecraft_version: Option<String>,
    pub file_hash: String,
    pub size_bytes: i64,
    pub upload_key: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Preview thumbnail for a world (from its first available capture)
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct WorldPreviewCapture {
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
}

/// World summary for list endpoints with aggregate counts
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct WorldListItem {
    #[serde(flatten)]
    pub world: World,
    pub latest_version: Option<WorldVersion>,
    pub scene_count: i64,
    pub version_count: i64,
    pub capture_count: i64,
    pub preview: Option<WorldPreviewCapture>,
}

/// World with its associated scenes and latest version
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct WorldWithDetails {
    #[serde(flatten)]
    pub world: World,
    pub scenes: Vec<Scene>,
    pub latest_version: Option<WorldVersion>,
}

/// Request to initiate a world upload (returns presigned URL)
#[derive(Debug, Deserialize)]
pub struct CreateWorldUploadRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    /// SHA256 hash with algorithm prefix (e.g., "sha256:abc123...")
    pub file_hash: String,
    pub file_size_bytes: i64,
}

/// Response with presigned URL for an upload
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct UploadResponse {
    pub upload_id: String,
    pub presigned_url: String,
    #[ts(type = "string")]
    pub expires_at: DateTime<Utc>,
}

/// Request to initiate a new version upload for an existing world
#[derive(Debug, Deserialize)]
pub struct CreateWorldVersionUploadRequest {
    /// SHA256 hash with algorithm prefix (e.g., "sha256:abc123...")
    pub file_hash: String,
    pub file_size_bytes: i64,
}

/// Request to complete an upload (world creation or version)
#[derive(Debug, Deserialize)]
pub struct CompleteUploadRequest {
    pub upload_id: String,
}

/// Internal request for creating a world record in the database
#[derive(Debug)]
pub struct CreateWorldRequest<'a> {
    pub name: &'a str,
    pub slug: &'a str,
    pub description: Option<&'a str>,
    pub minecraft_version: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorldRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}
