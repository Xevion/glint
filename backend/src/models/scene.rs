use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use ts_rs::TS;
use validator::{Validate, ValidationError};

use super::capture::CaptureWithContext;
use super::taxonomy::Tag;
use crate::id::{PendingSceneUploadId, SceneId, ScenePresetId, SceneVersionId};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct Scene {
    pub id: SceneId,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub dimension: String,
    pub active: bool,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

/// A specific revision of a Scene's config (position, camera, scene package, etc.)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct SceneVersion {
    pub id: SceneVersionId,
    pub scene_id: SceneId,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: f64,
    pub yaw: f64,
    pub time_of_day_ticks: i32,
    pub weather: String,
    pub weather_intensity: f64,
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
    pub package_url: Option<String>,
    pub package_hash: Option<String>,
    pub package_size_bytes: Option<i64>,
    pub minecraft_version: Option<String>,
    pub fov: i32,
    pub render_distance: i32,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

/// Scene with its latest version nested (for API responses that need config)
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SceneWithVersion {
    #[serde(flatten)]
    pub scene: Scene,
    pub version: SceneVersion,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct SceneListItem {
    #[serde(flatten)]
    pub scene: Scene,
    pub version: SceneVersion,
    pub tags: Vec<Tag>,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
    pub capture_count: i64,
}

/// A preset within a scene (time/weather/moon_phase variation)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct ScenePreset {
    pub id: ScenePresetId,
    pub scene_id: SceneId,
    pub name: String,
    pub slug: String,
    pub time_of_day_ticks: i32,
    pub weather: String,
    pub weather_intensity: f64,
    pub moon_phase: Option<i32>,
    pub sort_order: i32,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct SceneWithCaptures {
    #[serde(flatten)]
    pub scene: Scene,
    pub version: SceneVersion,
    pub presets: Vec<ScenePreset>,
    pub captures: Vec<CaptureWithContext>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct SceneListAdmin {
    #[serde(flatten)]
    pub scene: Scene,
    pub version: SceneVersion,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
    pub capture_count: i64,
}

fn validate_finite(value: f64) -> Result<(), ValidationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ValidationError::new("not_finite"))
    }
}

fn validate_weather(value: &str) -> Result<(), ValidationError> {
    match value {
        "clear" | "rain" | "thunder" => Ok(()),
        _ => Err(ValidationError::new("invalid_weather")),
    }
}

fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ValidationError::new("invalid_slug_format"));
    }
    Ok(())
}

/// Helper types for scene position and camera
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Position {
    #[validate(custom(function = "validate_finite"))]
    pub x: f64,
    #[validate(custom(function = "validate_finite"))]
    pub y: f64,
    #[validate(custom(function = "validate_finite"))]
    pub z: f64,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Camera {
    #[validate(custom(function = "validate_finite"))]
    pub yaw: f64,
    #[validate(custom(function = "validate_finite"))]
    pub pitch: f64,
}

/// Create scene request (from mod - includes name, no description/tags)
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSceneRequest {
    #[validate(length(min = 1, max = 100), custom(function = "validate_slug"))]
    pub slug: String,
    pub name: String,
    #[validate(nested)]
    pub position: Position,
    #[validate(nested)]
    pub camera: Camera,
    pub dimension: String,
    #[validate(range(min = 0, max = 24000))]
    pub time_of_day: i32,
    #[validate(custom(function = "validate_weather"))]
    pub weather: String,
    #[serde(default)]
    #[validate(range(min = 0.0, max = 1.0))]
    pub weather_intensity: f64,
    #[validate(range(min = 0, max = 7))]
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
}

/// Update scene request (from mod - no name/description/tags, only positioning)
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSceneRequest {
    #[validate(nested)]
    pub position: Position,
    #[validate(nested)]
    pub camera: Camera,
    pub dimension: String,
    #[validate(range(min = 0, max = 24000))]
    pub time_of_day: i32,
    #[validate(custom(function = "validate_weather"))]
    pub weather: String,
    #[serde(default)]
    #[validate(range(min = 0.0, max = 1.0))]
    pub weather_intensity: f64,
    #[validate(range(min = 0, max = 7))]
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct UpdateSceneMetadataRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Scene Package Upload Types
// ---------------------------------------------------------------------------

/// DB row for pending scene package uploads (initiate → confirm flow).
/// Internal type — not serialized to API responses.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PendingSceneUpload {
    pub id: String,
    pub scene_id: Option<String>,
    pub scene_name: Option<String>,
    pub scene_slug: Option<String>,
    pub scene_dimension: Option<String>,
    pub scene_description: Option<String>,
    pub minecraft_version: String,
    pub file_hash: String,
    pub size_bytes: i64,
    pub r2_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// POST /api/scenes — Initiate upload for a NEW scene (agent).
/// Creates a pending upload record with scene metadata, returns presigned URL.
#[derive(Debug, Deserialize, Validate)]
pub struct InitiateNewSceneUploadRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100), custom(function = "validate_slug"))]
    pub slug: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub dimension: String,
    #[validate(length(min = 1, max = 20))]
    pub minecraft_version: String,
    #[validate(length(min = 1))]
    pub file_hash: String,
    #[validate(range(min = 1))]
    pub size_bytes: i64,
}

/// POST /api/scenes/{slug}/versions — Initiate upload for an EXISTING scene (agent).
#[derive(Debug, Deserialize, Validate)]
pub struct InitiateVersionUploadRequest {
    #[validate(length(min = 1))]
    pub file_hash: String,
    #[validate(range(min = 1))]
    pub size_bytes: i64,
    #[validate(length(min = 1, max = 20))]
    pub minecraft_version: String,
}

/// Response from upload initiation endpoints.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct UploadInitiatedResponse {
    pub upload_id: PendingSceneUploadId,
    pub upload_url: String,
}

/// Environment settings within a scene version.
#[derive(Debug, Deserialize, Validate)]
pub struct Environment {
    #[validate(range(min = 0, max = 24000))]
    pub time_of_day_ticks: i32,
    #[validate(custom(function = "validate_weather"))]
    pub weather: String,
    #[serde(default)]
    #[validate(range(min = 0.0, max = 1.0))]
    pub weather_intensity: f64,
    #[validate(range(min = 0, max = 7))]
    pub moon_phase: Option<i32>,
}

/// POST /api/scenes/uploads/{upload_id}/complete — Confirm upload and create scene version.
#[derive(Debug, Deserialize, Validate)]
pub struct CompleteUploadRequest {
    #[validate(length(min = 1))]
    pub file_hash: String,
    #[validate(nested)]
    pub camera: Camera,
    #[validate(nested)]
    pub environment: Environment,
    #[validate(range(min = 30, max = 110))]
    pub fov: i32,
    #[validate(range(min = 2, max = 64))]
    pub render_distance: i32,
    pub biome: Option<String>,
    #[validate(nested)]
    pub position: Position,
}

/// Response from upload confirmation.
#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct CompleteUploadResponse {
    pub scene_id: SceneId,
    pub scene_version_id: SceneVersionId,
    pub preset_id: Option<ScenePresetId>,
}

// ---------------------------------------------------------------------------
// Scene Preset CRUD Types
// ---------------------------------------------------------------------------

/// POST /api/scenes/{slug}/presets — Create a new preset.
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePresetRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100), custom(function = "validate_slug"))]
    pub slug: String,
    #[validate(range(min = 0, max = 24000))]
    pub time_of_day_ticks: i32,
    #[validate(custom(function = "validate_weather"))]
    pub weather: String,
    #[serde(default)]
    #[validate(range(min = 0.0, max = 1.0))]
    pub weather_intensity: f64,
    #[validate(range(min = 0, max = 7))]
    pub moon_phase: Option<i32>,
}

/// PATCH /api/scenes/{slug}/presets/{preset_slug} — Update a preset.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePresetRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(range(min = 0, max = 24000))]
    pub time_of_day_ticks: Option<i32>,
    #[validate(custom(function = "validate_weather"))]
    pub weather: Option<String>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub weather_intensity: Option<f64>,
    // Note: COALESCE in the update query can't clear this to NULL.
    // If clearing moon_phase is needed, switch to double-option pattern.
    #[validate(range(min = 0, max = 7))]
    pub moon_phase: Option<i32>,
}

/// PATCH /api/scenes/{slug}/presets/reorder — Reorder presets.
#[derive(Debug, Deserialize, Validate)]
pub struct ReorderPresetsRequest {
    /// Ordered list of preset IDs. The sort_order of each preset
    /// is set to its index in this array.
    #[validate(length(min = 1, max = 100))]
    pub preset_ids: Vec<String>,
}
