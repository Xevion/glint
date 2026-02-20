use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::models::stats::Stats;

#[derive(SimpleObject, Debug, Clone)]
pub struct StatsNode {
    pub shader_count: i64,
    pub scene_count: i64,
    pub capture_count: i64,
    pub user_count: i64,
    pub run_count: i64,
    pub latest_capture_at: Option<DateTime<Utc>>,
}

impl From<Stats> for StatsNode {
    fn from(s: Stats) -> Self {
        Self {
            shader_count: s.shader_count,
            scene_count: s.scene_count,
            capture_count: s.capture_count,
            user_count: s.user_count,
            run_count: s.run_count,
            latest_capture_at: s.latest_capture_at,
        }
    }
}
