use std::time::Duration;

use aws_sdk_s3::Client as S3Client;
use sqlx::SqlitePool;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Interval between cleanup runs (10 minutes)
const CLEANUP_INTERVAL_SECS: u64 = 600;

/// Background task that cleans up expired pending uploads.
/// Runs every 10 minutes and removes:
/// - Expired pending_uploads records from the database
/// - Corresponding files from R2/S3 storage
pub async fn cleanup_expired_uploads(pool: SqlitePool, s3: Option<S3Client>, bucket: String) {
    info!(
        "Starting upload cleanup service (interval: {}s)",
        CLEANUP_INTERVAL_SECS
    );

    let mut ticker = interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));

    loop {
        ticker.tick().await;

        match run_cleanup(&pool, s3.as_ref(), &bucket).await {
            Ok(cleaned) => {
                if cleaned > 0 {
                    info!("Cleaned up {} expired pending upload(s)", cleaned);
                } else {
                    debug!("No expired uploads to clean up");
                }
            }
            Err(e) => {
                error!("Failed to clean up expired uploads: {}", e);
            }
        }
    }
}

/// Run a single cleanup pass
async fn run_cleanup(
    pool: &SqlitePool,
    s3: Option<&S3Client>,
    bucket: &str,
) -> anyhow::Result<usize> {
    // Find all expired pending uploads
    let expired: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT upload_id, upload_key 
        FROM pending_uploads 
        WHERE expires_at < datetime('now', 'utc')
        "#,
    )
    .fetch_all(pool)
    .await?;

    if expired.is_empty() {
        return Ok(0);
    }

    let mut cleaned = 0;

    for (upload_id, upload_key) in &expired {
        // Delete from R2/S3 if configured
        if let Some(client) = s3 {
            match client
                .delete_object()
                .bucket(bucket)
                .key(upload_key)
                .send()
                .await
            {
                Ok(_) => {
                    debug!("Deleted expired upload file: {}", upload_key);
                }
                Err(e) => {
                    // Log but continue - file may already be gone
                    warn!(
                        "Failed to delete expired upload file {} (may already be deleted): {}",
                        upload_key, e
                    );
                }
            }
        }

        // Delete from database
        match sqlx::query("DELETE FROM pending_uploads WHERE upload_id = ?1")
            .bind(upload_id)
            .execute(pool)
            .await
        {
            Ok(_) => {
                debug!("Deleted expired pending upload record: {}", upload_id);
                cleaned += 1;
            }
            Err(e) => {
                error!(
                    "Failed to delete pending upload record {}: {}",
                    upload_id, e
                );
            }
        }
    }

    Ok(cleaned)
}
