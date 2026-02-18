use schemars::JsonSchema;
use serde_with::skip_serializing_none;
use ts_rs::TS;

use crate::id::{
    SceneId, ScenePresetId, SceneVersionId, ShaderId, ShaderVersionId, ShaderVersionProfileId,
};

/// Raw row returned by the work-items SQL query.
/// Internal only — used in `repo/work.rs`, then converted to [`WorkItem`].
#[derive(Debug, sqlx::FromRow)]
pub struct WorkItemRow {
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
    pub preset_id: Option<ScenePresetId>,
    pub preset_name: Option<String>,
    pub preset_slug: Option<String>,
    pub preset_time_of_day_ticks: Option<i32>,
    pub preset_weather: Option<String>,
    pub preset_weather_intensity: Option<f64>,
    pub preset_moon_phase: Option<i32>,
    pub package_url: Option<String>,
    pub package_hash: Option<String>,
    pub package_size_bytes: Option<i64>,
    pub scene_minecraft_version: Option<String>,
    pub scene_fov: i32,
    pub scene_render_distance: i32,
    pub scene_version_id: Option<SceneVersionId>,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub profile_name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, serde::Serialize, JsonSchema, TS)]
#[ts(export, optional_fields)]
pub struct WorkShader {
    pub version_id: ShaderVersionId,
    pub id: ShaderId,
    pub slug: String,
    pub name: String,
    pub version: String,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub profile_name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, serde::Serialize, JsonSchema, TS)]
#[ts(export, optional_fields)]
pub struct WorkScene {
    pub id: SceneId,
    pub slug: String,
    pub name: String,
    pub dimension: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub time_of_day_ticks: i32,
    pub weather: String,
    pub weather_intensity: f64,
    pub moon_phase: Option<i32>,
    pub biome: Option<String>,
    pub fov: i32,
    pub render_distance: i32,
    pub minecraft_version: Option<String>,
    pub version_id: Option<SceneVersionId>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Serialize, JsonSchema, TS)]
#[ts(export, optional_fields)]
pub struct WorkPreset {
    pub id: ScenePresetId,
    pub name: String,
    pub slug: String,
    pub time_of_day_ticks: Option<i32>,
    pub weather: Option<String>,
    pub weather_intensity: Option<f64>,
    pub moon_phase: Option<i32>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Serialize, JsonSchema, TS)]
#[ts(export, optional_fields)]
pub struct WorkPackage {
    pub url: String,
    pub hash: String,
    pub size_bytes: Option<i64>,
}

/// A single work item: one (shader_version, scene, preset, profile) tuple to capture
#[skip_serializing_none]
#[derive(Debug, Clone, serde::Serialize, JsonSchema, TS)]
#[ts(export, optional_fields)]
pub struct WorkItem {
    pub shader: WorkShader,
    pub scene: WorkScene,
    pub preset: Option<WorkPreset>,
    pub scene_package: Option<WorkPackage>,
}

impl From<WorkItemRow> for WorkItem {
    fn from(row: WorkItemRow) -> Self {
        let preset = row.preset_id.map(|id| WorkPreset {
            id,
            name: row.preset_name.expect("preset_id set but name missing"),
            slug: row.preset_slug.expect("preset_id set but slug missing"),
            time_of_day_ticks: row.preset_time_of_day_ticks,
            weather: row.preset_weather,
            weather_intensity: row.preset_weather_intensity,
            moon_phase: row.preset_moon_phase,
        });

        let scene_package = row.package_url.map(|url| WorkPackage {
            url,
            hash: row.package_hash.expect("package_url set but hash missing"),
            size_bytes: row.package_size_bytes,
        });

        WorkItem {
            shader: WorkShader {
                version_id: row.shader_version_id,
                id: row.shader_id,
                slug: row.shader_slug,
                name: row.shader_name,
                version: row.version,
                download_url: row.download_url,
                file_hash: row.file_hash,
                profile_id: row.profile_id,
                profile_name: row.profile_name,
            },
            scene: WorkScene {
                id: row.scene_id,
                slug: row.scene_slug,
                name: row.scene_name,
                dimension: row.scene_dimension,
                x: row.scene_x,
                y: row.scene_y,
                z: row.scene_z,
                yaw: row.scene_yaw,
                pitch: row.scene_pitch,
                time_of_day_ticks: row.scene_time_of_day_ticks,
                weather: row.scene_weather,
                weather_intensity: row.scene_weather_intensity,
                moon_phase: row.scene_moon_phase,
                biome: row.scene_biome,
                fov: row.scene_fov,
                render_distance: row.scene_render_distance,
                minecraft_version: row.scene_minecraft_version,
                version_id: row.scene_version_id,
            },
            preset,
            scene_package,
        }
    }
}
