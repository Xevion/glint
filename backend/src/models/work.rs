use schemars::JsonSchema;
use serde_with::skip_serializing_none;
use ts_rs::TS;

use crate::id::{
    SceneId, SceneVersionId, ShaderId, ShaderVersionId, ShaderVersionProfileId, WorldId,
    WorldVersionId,
};

/// A single work item: one (shader_version, scene, profile) triple to capture
#[skip_serializing_none]
#[derive(Debug, sqlx::FromRow, serde::Serialize, JsonSchema, TS)]
#[ts(export, optional_fields)]
pub struct WorkItem {
    pub shader_version_id: ShaderVersionId,
    pub shader_id: ShaderId,
    pub shader_slug: String,
    pub shader_name: String,
    pub version: String,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub scene_id: SceneId,
    pub scene_slug: String,
    pub scene_name: String,
    pub scene_dimension: String,
    pub scene_x: f64,
    pub scene_y: f64,
    pub scene_z: f64,
    pub scene_yaw: f64,
    pub scene_pitch: f64,
    pub scene_time_of_day_ticks: i32,
    pub scene_weather: String,
    pub scene_weather_intensity: f64,
    pub scene_moon_phase: Option<i32>,
    pub scene_biome: Option<String>,
    pub world_id: WorldId,
    pub world_slug: String,
    pub world_name: String,
    pub world_file_url: Option<String>,
    pub world_file_hash: Option<String>,
    pub world_size_bytes: Option<i64>,
    pub world_version_id: Option<WorldVersionId>,
    pub scene_version_id: Option<SceneVersionId>,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub profile_name: Option<String>,
}
