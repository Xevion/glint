use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::id::{SceneId, SceneVersionId};
use crate::models::SceneVersion;

#[derive(SimpleObject, Debug, Clone)]
pub struct SceneVersionNode {
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
    pub fov: i32,
    pub render_distance: i32,
    pub created_at: DateTime<Utc>,
}

impl From<SceneVersion> for SceneVersionNode {
    fn from(v: SceneVersion) -> Self {
        Self {
            id: v.id,
            scene_id: v.scene_id,
            x: v.x,
            y: v.y,
            z: v.z,
            pitch: v.pitch,
            yaw: v.yaw,
            time_of_day_ticks: v.time_of_day_ticks,
            weather: v.weather,
            weather_intensity: v.weather_intensity,
            moon_phase: v.moon_phase,
            biome: v.biome,
            fov: v.fov,
            render_distance: v.render_distance,
            created_at: v.created_at,
        }
    }
}
