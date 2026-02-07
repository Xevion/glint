pub mod agent;
pub mod device;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

/// Downloadable world files containing scenes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct World {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub file_url: Option<String>,
    pub file_hash: Option<String>,
    pub size_bytes: Option<i64>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// Tracks pending world uploads (presigned URL workflow)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PendingUpload {
    pub upload_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub file_hash: String,
    pub size_bytes: i64,
    pub upload_key: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Shader pack identity (not version-specific)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Shader {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
    pub icon_url: Option<String>,
    pub source_url: Option<String>,
    pub license_id: Option<String>,
    pub upstream_downloads: Option<i64>,
    #[ts(type = "string | null")]
    pub upstream_updated_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub last_synced_at: Option<DateTime<Utc>>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// Specific release of a shader pack
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct ShaderVersion {
    pub id: String,
    pub shader_id: String,
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub curseforge_file_id: Option<i32>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub file_size: Option<i64>,
    pub game_versions: Option<String>,
    pub release_channel: Option<String>,
    /// JSON array of profile names, discovered after first capture
    pub supported_profiles: Option<String>,
    #[ts(type = "string | null")]
    pub upstream_published_at: Option<DateTime<Utc>>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    pub capture_failure_count: i32,
    pub last_capture_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
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
    pub time_of_day_ticks: i32,
    pub weather: String,
    pub weather_intensity: f64,
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
    pub definition_json: Option<String>,
    pub active: bool,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Capture {
    pub id: String,
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
    pub image_url: Option<String>,
    pub image_path: Option<String>,
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
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    #[ts(type = "string | null")]
    pub captured_at: Option<DateTime<Utc>>,
    pub status: String,
    pub error_message: Option<String>,
    pub thumbhash: Option<String>,
    pub outdated: bool,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureRun {
    pub id: String,
    pub agent_id: Option<String>,
    #[ts(type = "string")]
    pub started_at: DateTime<Utc>,
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub skipped_items: i32,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureRunItem {
    pub id: String,
    pub run_id: String,
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
    pub status: String,
    pub capture_id: Option<String>,
    pub error_message: Option<String>,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
    #[ts(type = "string | null")]
    pub started_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Capture run item with denormalized shader/scene info for API responses
#[derive(Debug, Clone, Serialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureRunItemWithContext {
    pub id: String,
    pub run_id: String,
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
    pub status: String,
    pub capture_id: Option<String>,
    pub error_message: Option<String>,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
    #[ts(type = "string | null")]
    pub started_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
    // Denormalized context
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_version: String,
    pub scene_name: String,
}

/// Discord OAuth user
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct User {
    pub id: i32,
    pub discord_id: String,
    pub discord_username: String,
    pub discord_avatar: Option<String>,
    pub role: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// Database-backed session (with in-memory cache at runtime)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub token: String,
    pub user_id: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Shader style category (realistic, fantasy, cartoon, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Category {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

/// Shader technical feature (volumetric, PBR, ray tracing, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Feature {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

/// Scene tag (indoor, sunset, water, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Tag {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

impl Scene {
    /// Builds a definition JSON matching the Minecraft mod's Scene data class format.
    /// Uses the explicit `definition_json` column if set, otherwise constructs it
    /// from the individual columns.
    pub fn build_definition_json(&self) -> String {
        if let Some(ref json) = self.definition_json
            && json != "{}"
        {
            return json.clone();
        }

        let weather = self.weather.to_uppercase();

        let mut json = serde_json::json!({
            "id": self.slug,
            "name": self.name,
            "position": {
                "x": self.x,
                "y": self.y,
                "z": self.z
            },
            "camera": {
                "yaw": self.yaw,
                "pitch": self.pitch
            },
            "timeOfDay": self.time_of_day_ticks,
            "dimension": self.dimension,
            "weather": weather,
            "weatherIntensity": self.weather_intensity
        });

        if let Some(ref biome) = self.biome {
            json["biome"] = serde_json::Value::String(biome.clone());
        }

        if let Some(moon_phase) = self.moon_phase {
            json["moonPhase"] = serde_json::Value::Number(moon_phase.into());
        }

        json.to_string()
    }
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderListItem {
    #[serde(flatten)]
    pub shader: Shader,
    pub authors: Vec<ShaderAuthor>,
    pub categories: Vec<Category>,
    pub features: Vec<Feature>,
    pub latest_version: Option<String>,
    pub game_versions: Option<String>,
    pub thumbnail_url: Option<String>,
    pub thumbhash: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SceneListItem {
    #[serde(flatten)]
    pub scene: Scene,
    pub tags: Vec<Tag>,
    pub thumbnail_url: Option<String>,
    pub thumbhash: Option<String>,
    pub capture_count: i64,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderWithVersions {
    #[serde(flatten)]
    pub shader: Shader,
    pub versions: Vec<ShaderVersion>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderWithCaptures {
    #[serde(flatten)]
    pub shader: Shader,
    pub versions: Vec<ShaderVersion>,
    pub captures: Vec<CaptureWithContext>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SceneWithCaptures {
    #[serde(flatten)]
    pub scene: Scene,
    pub world: Option<World>,
    pub captures: Vec<CaptureWithContext>,
}

/// Capture with denormalized shader/version info for API responses
#[derive(Debug, Serialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureWithContext {
    pub id: String,
    pub scene_id: String,
    pub shader_slug: String,
    pub shader_name: String,
    pub shader_version: String,
    pub profile: Option<String>,
    pub image_path: Option<String>,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
    #[ts(type = "string | null")]
    pub captured_at: Option<DateTime<Utc>>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    // Run context
    pub run_id: Option<String>,
    pub run_status: Option<String>,
    // Shader author
    pub shader_author: Option<String>,
    // Scene context
    pub scene_name: Option<String>,
    pub scene_slug: Option<String>,
}

/// World with its associated scenes
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct WorldWithScenes {
    #[serde(flatten)]
    pub world: World,
    pub scenes: Vec<Scene>,
}

/// Paginated captures response envelope
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PaginatedCaptures {
    pub items: Vec<CaptureWithContext>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// Shader author from upstream platform
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct ShaderAuthor {
    pub id: String,
    pub shader_id: String,
    pub name: String,
    pub url: Option<String>,
    pub platform: String,
}

// Platform adoption types

/// Request to preview or adopt a shader from a platform URL
#[derive(Debug, Deserialize)]
pub struct AdoptShaderRequest {
    pub url: String,
}

/// Request to link an additional platform to an existing shader
#[derive(Debug, Deserialize)]
pub struct LinkShaderRequest {
    pub url: String,
}

/// Preview response before confirming adoption
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AdoptPreviewResponse {
    pub platform: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub version_count: usize,
    pub authors: Vec<AdoptPreviewAuthor>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AdoptPreviewAuthor {
    pub name: String,
    pub url: Option<String>,
}

// Admin-only operations: create/update shaders, worlds, scenes

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

/// Response with presigned URL for world upload
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreateWorldUploadResponse {
    pub upload_id: String,
    pub presigned_url: String,
    #[ts(type = "string")]
    pub expires_at: DateTime<Utc>,
}

/// Request to complete a world upload
#[derive(Debug, Deserialize)]
pub struct CompleteWorldUploadRequest {
    pub upload_id: String,
}

/// Internal request for creating a world record in the database
#[derive(Debug)]
pub struct CreateWorldRequest<'a> {
    pub name: &'a str,
    pub slug: &'a str,
    pub description: Option<&'a str>,
    pub minecraft_version: &'a str,
    pub file_url: &'a str,
    pub file_hash: &'a str,
    pub size_bytes: i64,
}

/// Helper types for scene position and camera
#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Camera {
    pub yaw: f64,
    pub pitch: f64,
}

/// Create scene request (from mod - includes name, no description/tags)
#[derive(Debug, Deserialize)]
pub struct CreateSceneRequest {
    pub world_id: String,
    pub slug: String,
    pub name: String,
    pub position: Position,
    pub camera: Camera,
    pub dimension: String,
    #[serde(rename = "timeOfDay")]
    pub time_of_day: i32,
    pub weather: String,
    #[serde(rename = "weatherIntensity", default)]
    pub weather_intensity: f64,
    #[serde(rename = "moonPhase")]
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
}

/// Update scene request (from mod - no name/description/tags, only positioning)
#[derive(Debug, Deserialize)]
pub struct UpdateSceneRequest {
    pub world_id: String,
    pub position: Position,
    pub camera: Camera,
    pub dimension: String,
    #[serde(rename = "timeOfDay")]
    pub time_of_day: i32,
    pub weather: String,
    #[serde(rename = "weatherIntensity", default)]
    pub weather_intensity: f64,
    #[serde(rename = "moonPhase")]
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
}

// Admin dashboard request/response types

#[derive(Debug, Deserialize)]
pub struct UpdateShaderRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorldRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSceneMetadataRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct UserWithSessions {
    #[serde(flatten)]
    pub user: User,
    pub sessions: Vec<SessionInfo>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SessionInfo {
    pub token_prefix: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SceneWithWorld {
    #[serde(flatten)]
    pub scene: Scene,
    pub world_name: Option<String>,
    pub world_slug: Option<String>,
}

// Platform search types

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderSearchResult {
    pub platform: String,
    pub platform_id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub author: String,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub platform_url: String,
    /// Present when this shader has already been adopted into Glint
    pub adopted: Option<ShaderAdopted>,
}

/// Reference to an already-adopted shader in Glint
#[derive(Debug, Clone, Serialize, FromRow, TS)]
#[ts(export)]
pub struct ShaderAdopted {
    pub id: String,
    pub slug: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderSearchResponse {
    pub results: Vec<ShaderSearchResult>,
    pub total_modrinth: u32,
    pub total_curseforge: Option<u32>,
}

/// Sort order for shader browsing (when no search query is provided)
#[derive(Debug, Default, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ShaderSearchSort {
    #[default]
    Popular,
    Recent,
}

#[derive(Debug, Deserialize)]
pub struct ShaderSearchRequest {
    pub query: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    /// Sort order for browse mode (ignored when query is provided)
    pub sort: Option<ShaderSearchSort>,
}
