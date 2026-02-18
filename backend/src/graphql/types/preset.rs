use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::id::{SceneId, ScenePresetId};
use crate::models::ScenePreset;

#[derive(SimpleObject, Debug, Clone)]
pub struct PresetNode {
    pub id: ScenePresetId,
    pub scene_id: SceneId,
    pub name: String,
    pub slug: String,
    pub time_of_day_ticks: i32,
    pub weather: String,
    pub weather_intensity: f64,
    pub moon_phase: Option<i32>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ScenePreset> for PresetNode {
    fn from(p: ScenePreset) -> Self {
        Self {
            id: p.id,
            scene_id: p.scene_id,
            name: p.name,
            slug: p.slug,
            time_of_day_ticks: p.time_of_day_ticks,
            weather: p.weather,
            weather_intensity: p.weather_intensity,
            moon_phase: p.moon_phase,
            sort_order: p.sort_order,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
