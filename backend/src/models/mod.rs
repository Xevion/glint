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
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    /// JSON array of profile names, discovered after first capture
    pub supported_profiles: Option<String>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
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
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    #[ts(type = "string | null")]
    pub captured_at: Option<DateTime<Utc>>,
    pub status: String,
    pub error_message: Option<String>,
    pub outdated: bool,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
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
    #[ts(type = "string | null")]
    pub claimed_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub last_heartbeat: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub started_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
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
    pub screenshot_path: Option<String>,
    pub screenshot_url: Option<String>,
    #[ts(type = "string | null")]
    pub captured_at: Option<DateTime<Utc>>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
}

// Agent API types are now in the shared crate (glint-shared)

// Admin-only operations: create/update shaders, worlds, scenes, jobs

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

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub shader_version_id: String,
    pub scene_ids: Vec<String>,
    pub profiles: Option<Vec<String>>,
    pub priority: Option<i32>,
}
