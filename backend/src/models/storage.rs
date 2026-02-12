use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct StorageStats {
    pub total_bytes: i64,
    pub capture_count: i64,
    pub avg_bytes: i64,
    pub missing_count: i64,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct StorageBucket {
    #[serde(with = "chrono::serde::ts_seconds")]
    #[ts(type = "number")]
    pub date: DateTime<Utc>,
    pub cumulative_bytes: i64,
    pub cumulative_count: i64,
    pub bucket_bytes: i64,
}
