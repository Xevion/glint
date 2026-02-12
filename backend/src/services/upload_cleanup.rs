use std::time::Duration;

use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use tokio::time::interval;
use tracing::{debug, error, trace, warn};

use crate::repo::PendingUploadRepo;
use crate::services::lifecycle::ServiceContext;

/// Interval between cleanup runs (10 minutes)
const CLEANUP_INTERVAL_SECS: u64 = 600;

/// Background task that cleans up expired pending uploads.
/// Runs every 10 minutes and removes:
/// - Expired pending_uploads records from the database
/// - Corresponding files from R2/S3 storage
pub async fn cleanup_expired_uploads(
    ctx: ServiceContext,
    pool: PgPool,
    s3: Option<S3Client>,
    bucket: String,
) {
    let mut ticker = interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));

    while ctx.tick(&mut ticker).await {
        match run_cleanup(&pool, s3.as_ref(), &bucket).await {
            Ok(cleaned) => {
                if cleaned > 0 {
                    debug!(count = cleaned, "Cleaned up expired uploads");
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to clean up expired uploads");
            }
        }
    }
}

/// Run a single cleanup pass
async fn run_cleanup(pool: &PgPool, s3: Option<&S3Client>, bucket: &str) -> anyhow::Result<usize> {
    // Find all expired pending uploads
    let expired = PendingUploadRepo::list_expired(pool).await?;

    if expired.is_empty() {
        return Ok(0);
    }

    let mut cleaned = 0;

    for record in &expired {
        // Delete from R2/S3 if configured
        if let Some(client) = s3 {
            match client
                .delete_object()
                .bucket(bucket)
                .key(&record.upload_key)
                .send()
                .await
            {
                Ok(_) => {
                    trace!(key = %record.upload_key, "Deleted expired upload file");
                }
                Err(e) => {
                    // Log but continue - file may already be gone
                    warn!(key = %record.upload_key, error = %e, "Failed to delete expired upload file");
                }
            }
        }

        // Delete from database
        match PendingUploadRepo::delete_expired_by_id(pool, &record.upload_id).await {
            Ok(true) => {
                trace!(upload_id = %record.upload_id, "Deleted expired pending upload record");
                cleaned += 1;
            }
            Ok(false) => {
                trace!(upload_id = %record.upload_id, "Pending upload already deleted");
            }
            Err(e) => {
                error!(upload_id = %record.upload_id, error = %e, "Failed to delete pending upload record");
            }
        }
    }

    Ok(cleaned)
}
