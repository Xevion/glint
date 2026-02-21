use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};

use crate::models::storage::{StorageBucket, StorageStats};

/// Aggregate storage statistics for completed captures.
#[derive(SimpleObject, Debug, Clone)]
pub struct StorageStatsNode {
    pub total_bytes: i64,
    pub capture_count: i64,
    pub avg_bytes: i64,
    pub missing_count: i64,
}

impl From<StorageStats> for StorageStatsNode {
    fn from(s: StorageStats) -> Self {
        Self {
            total_bytes: s.total_bytes as i64,
            capture_count: s.capture_count as i64,
            avg_bytes: s.avg_bytes as i64,
            missing_count: s.missing_count as i64,
        }
    }
}

/// A single time-series bucket for storage growth.
#[derive(SimpleObject, Debug, Clone)]
pub struct StorageBucketNode {
    pub date: DateTime<Utc>,
    pub cumulative_bytes: i64,
    pub cumulative_count: i64,
    pub bucket_bytes: i64,
}

impl From<StorageBucket> for StorageBucketNode {
    fn from(b: StorageBucket) -> Self {
        Self {
            date: b.date,
            cumulative_bytes: b.cumulative_bytes as i64,
            cumulative_count: b.cumulative_count as i64,
            bucket_bytes: b.bucket_bytes as i64,
        }
    }
}
