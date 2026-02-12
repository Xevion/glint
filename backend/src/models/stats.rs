use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_with::skip_serializing_none;
use ts_rs::TS;

/// Lightweight aggregate statistics for the index page.
#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct Stats {
    pub shader_count: i64,
    pub scene_count: i64,
    pub capture_count: i64,
    #[ts(as = "Option<String>")]
    pub latest_capture_at: Option<DateTime<Utc>>,
}
