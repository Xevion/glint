use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

// --- Audit types ---

/// A single object found in R2 during audit
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AuditObject {
    pub key: String,
    pub size: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    #[ts(type = "number")]
    pub last_modified: DateTime<Utc>,
    /// The R2 key prefix (e.g. "captures", "packages", "_uploads")
    pub prefix: String,
}

/// A DB row that references a missing or mismatched R2 object
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AuditReference {
    pub table: String,
    pub row_id: String,
    pub stored_url: String,
    /// For missing: same as stored_url. For mismatch: what the URL should be.
    pub expected_url: Option<String>,
}

/// Full audit result returned by POST /api/admin/storage/audit
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct StorageAuditResult {
    /// Objects in R2 with no matching DB row
    pub orphaned: Vec<AuditObject>,
    /// Objects under _uploads/ (orphaned staging files)
    pub stale_staging: Vec<AuditObject>,
    /// DB rows referencing R2 URLs where the object doesn't exist in R2
    pub missing: Vec<AuditReference>,
    /// Objects under unrecognized prefixes
    pub unknown_prefix: Vec<AuditObject>,
    /// DB rows where stored URL doesn't match what would be generated from the key
    pub url_mismatches: Vec<AuditReference>,
    /// Summary statistics
    pub summary: AuditSummary,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AuditSummary {
    pub total_r2_objects: i64,
    pub total_r2_bytes: i64,
    pub total_db_references: i64,
    pub orphaned_count: i64,
    pub orphaned_bytes: i64,
    pub stale_staging_count: i64,
    pub missing_count: i64,
    pub unknown_prefix_count: i64,
    pub url_mismatch_count: i64,
}

/// Request body for POST /api/admin/storage/cleanup
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct StorageCleanupRequest {
    /// R2 keys to delete (max 200)
    pub keys: Vec<String>,
}

/// Result of a single key deletion attempt
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CleanupKeyResult {
    pub key: String,
    pub status: CleanupKeyStatus,
    /// Reason if skipped
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub enum CleanupKeyStatus {
    Deleted,
    Skipped,
    Failed,
}

/// Response from POST /api/admin/storage/cleanup
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct StorageCleanupResult {
    pub results: Vec<CleanupKeyResult>,
    pub deleted_count: i64,
    pub skipped_count: i64,
    pub failed_count: i64,
    pub freed_bytes: i64,
}
