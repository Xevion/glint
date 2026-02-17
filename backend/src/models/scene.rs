use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use ts_rs::TS;
use validator::{Validate, ValidationError};

use super::capture::CaptureWithContext;
use super::taxonomy::Tag;
use crate::id::{SceneId, ScenePresetId, SceneVersionId};

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
