use std::collections::HashSet;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use chrono::{DateTime, TimeZone, Utc};
use custom_debug_derive::Debug as CustomDebug;
use serde::Deserialize;
use tracing::{instrument, warn};

use crate::auth::AdminUser;
use crate::config::R2Config;
use crate::error::{AppError, AppResult};
use crate::models::{
    AuditObject, AuditReference, AuditSummary, CleanupKeyResult, CleanupKeyStatus,
    StorageAuditResult, StorageBucket, StorageCleanupRequest, StorageCleanupResult, StorageStats,
};
use crate::repo::{CaptureRepo, StorageRepo};
use crate::state::AppState;

/// Known R2 key prefixes.
const KNOWN_PREFIXES: &[&str] = &["captures", "packages", "backgrounds", "_uploads"];

#[derive(CustomDebug, Deserialize)]
struct GrowthParams {
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    days: Option<i32>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    interval_hours: Option<i32>,
}

/// Resolve a DB reference to an R2 object key.
/// References that are already S3 keys (captures, backgrounds) are returned as-is.
/// References that are full URLs (scene_versions.package_url) are converted via `key_from_url`.
fn resolve_key(reference: &str, r2_config: &R2Config) -> String {
    if reference.contains("://") {
        r2_config.key_from_url(reference)
    } else {
        reference.to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(storage_stats))
        .route("/growth", get(storage_growth))
        .route("/audit", post(storage_audit))
        .route("/cleanup", post(storage_cleanup))
}

/// GET /api/admin/storage/stats — Aggregate storage statistics
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn storage_stats(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<StorageStats>> {
    let stats = CaptureRepo::storage_stats(state.db()).await?;
    Ok(Json(stats))
}

/// GET /api/admin/storage/growth — Cumulative storage growth with gap-filled time series
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn storage_growth(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<GrowthParams>,
) -> AppResult<Json<Vec<StorageBucket>>> {
    let days = params.days.unwrap_or(90).clamp(1, 365);
    let interval_hours = params.interval_hours.unwrap_or(1).clamp(1, 24);
    let buckets = CaptureRepo::storage_growth(state.db(), days, interval_hours).await?;
    Ok(Json(buckets))
}

/// Extract the prefix from an R2 key (the part before the first `/`).
fn prefix_of(key: &str) -> &str {
    key.split('/').next().unwrap_or("")
}

/// POST /api/admin/storage/audit — Compare R2 objects against DB references
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn storage_audit(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<StorageAuditResult>> {
    let s3 = state
        .s3()
        .ok_or_else(|| AppError::ServiceUnavailable("S3/R2 storage is not configured".into()))?;

    let bucket = state.config().r2.bucket.as_deref().unwrap_or("glint");

    // 1. List all R2 objects via paginated ListObjectsV2
    let mut r2_objects: Vec<(String, u64, DateTime<Utc>)> = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = s3.list_objects_v2().bucket(bucket);
        if let Some(ref token) = continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to list R2 objects: {e}")))?;

        for obj in resp.contents() {
            let key = obj.key().unwrap_or_default().to_string();
            let size = obj.size().unwrap_or(0) as u64;
            let last_modified = obj
                .last_modified()
                .and_then(|dt| Utc.timestamp_opt(dt.secs(), dt.subsec_nanos()).single())
                .unwrap_or_else(Utc::now);
            r2_objects.push((key, size, last_modified));
        }

        match resp.next_continuation_token() {
            Some(token) => continuation_token = Some(token.to_string()),
            None => break,
        }
    }

    // Build a set of all R2 keys for quick lookup
    let r2_key_set: HashSet<&str> = r2_objects.iter().map(|(k, _, _)| k.as_str()).collect();

    // 2. Get all DB references and resolve to R2 keys
    let db_refs = StorageRepo::all_references(state.db())
        .await
        .map_err(AppError::Internal)?;

    let r2_config = &state.config().r2;
    let mut referenced_keys: HashSet<String> = HashSet::new();

    for db_ref in &db_refs {
        referenced_keys.insert(resolve_key(&db_ref.reference, r2_config));
    }

    // 3. Categorize R2 objects
    let mut orphaned = Vec::new();
    let mut stale_staging = Vec::new();
    let mut unknown_prefix = Vec::new();
    let mut total_r2_bytes: u64 = 0;
    let mut orphaned_bytes: u64 = 0;

    for (key, size, last_modified) in &r2_objects {
        total_r2_bytes += size;

        if referenced_keys.contains(key.as_str()) {
            continue;
        }

        let prefix = prefix_of(key);
        let is_known = KNOWN_PREFIXES.contains(&prefix);

        let audit_obj = AuditObject {
            key: key.clone(),
            size: *size,
            last_modified: *last_modified,
            prefix: prefix.to_string(),
        };

        if prefix == "_uploads" {
            stale_staging.push(audit_obj);
        } else if is_known {
            orphaned_bytes += size;
            orphaned.push(audit_obj);
        } else {
            unknown_prefix.push(audit_obj);
        }
    }

    // 4. Check for missing R2 objects (DB references with no R2 object)
    let mut missing = Vec::new();
    for db_ref in &db_refs {
        let key = resolve_key(&db_ref.reference, r2_config);

        if !r2_key_set.contains(key.as_str()) {
            missing.push(AuditReference {
                table: db_ref.table.to_string(),
                row_id: db_ref.row_id.clone(),
                stored_url: db_ref.reference.clone(),
                expected_url: None,
            });
        }
    }

    let summary = AuditSummary {
        total_r2_objects: r2_objects.len() as u64,
        total_r2_bytes,
        total_db_references: db_refs.len() as u64,
        orphaned_count: orphaned.len() as u64,
        orphaned_bytes,
        stale_staging_count: stale_staging.len() as u64,
        missing_count: missing.len() as u64,
        unknown_prefix_count: unknown_prefix.len() as u64,
        url_mismatch_count: 0,
    };

    Ok(Json(StorageAuditResult {
        orphaned,
        stale_staging,
        missing,
        unknown_prefix,
        url_mismatches: Vec::new(),
        summary,
    }))
}

/// POST /api/admin/storage/cleanup — Delete unreferenced R2 keys
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn storage_cleanup(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<StorageCleanupRequest>,
) -> AppResult<Json<StorageCleanupResult>> {
    const MAX_KEYS: usize = 200;

    if request.keys.len() > MAX_KEYS {
        return Err(AppError::BadRequest(format!(
            "Too many keys: {} exceeds maximum of {MAX_KEYS}",
            request.keys.len()
        )));
    }

    let s3 = state
        .s3()
        .ok_or_else(|| AppError::ServiceUnavailable("S3/R2 storage is not configured".into()))?;

    let bucket = state.config().r2.bucket.as_deref().unwrap_or("glint");
    let r2_config = &state.config().r2;

    // Build referenced key set for safety check
    let db_refs = StorageRepo::all_references(state.db())
        .await
        .map_err(AppError::Internal)?;

    let referenced_keys: HashSet<String> = db_refs
        .iter()
        .map(|r| resolve_key(&r.reference, r2_config))
        .collect();

    let mut results = Vec::with_capacity(request.keys.len());
    let mut deleted_count: u64 = 0;
    let mut skipped_count: u64 = 0;
    let mut failed_count: u64 = 0;
    let mut freed_bytes: u64 = 0;

    for key in &request.keys {
        // Safety: re-check if key is still unreferenced
        if referenced_keys.contains(key.as_str()) {
            skipped_count += 1;
            results.push(CleanupKeyResult {
                key: key.clone(),
                status: CleanupKeyStatus::Skipped,
                reason: Some("Key is now referenced by database".into()),
            });
            continue;
        }

        // Get object size before deleting
        let size = match s3.head_object().bucket(bucket).key(key).send().await {
            Ok(head) => head.content_length().unwrap_or(0) as u64,
            Err(e) => {
                warn!(key = %key, error = %e, "HeadObject failed during cleanup");
                0
            }
        };

        // Delete the object
        match s3.delete_object().bucket(bucket).key(key).send().await {
            Ok(_) => {
                freed_bytes += size;
                deleted_count += 1;
                results.push(CleanupKeyResult {
                    key: key.clone(),
                    status: CleanupKeyStatus::Deleted,
                    reason: None,
                });
            }
            Err(e) => {
                failed_count += 1;
                results.push(CleanupKeyResult {
                    key: key.clone(),
                    status: CleanupKeyStatus::Failed,
                    reason: Some(format!("Delete failed: {e}")),
                });
            }
        }
    }

    Ok(Json(StorageCleanupResult {
        results,
        deleted_count,
        skipped_count,
        failed_count,
        freed_bytes,
    }))
}
