use std::time::Duration;

use sqlx::PgPool;
use tokio::time::interval;
use tracing::{debug, error, info, trace, warn};

use crate::repo::CaptureRepo;
use crate::services::lifecycle::ServiceContext;

/// How often the integrity sweep runs (30 minutes)
const SWEEP_INTERVAL_SECS: u64 = 30 * 60;

/// Captures in 'uploading' status older than this are considered orphaned (10 minutes)
const UPLOADING_ORPHAN_THRESHOLD_SECS: i64 = 10 * 60;

/// Completed captures must have metadata populated within this window (1 hour).
/// If still missing after this, the sweep will verify the image exists in R2.
const METADATA_GRACE_PERIOD_SECS: i64 = 60 * 60;

/// Background task that periodically verifies capture data integrity.
///
/// Two classes of problems are detected:
///
/// 1. **Orphaned uploads** — captures stuck in `uploading` status past a threshold,
///    where the mod claimed them but never confirmed the upload. The sweep does a
///    HEAD request against R2: if the image exists, it completes the capture; if not,
///    it marks it failed.
///
/// 2. **Unverified completions** — captures marked `completed` but still missing
///    thumbhash or file_size metadata after the grace period. This catches cases
///    where the metadata worker failed due to CDN propagation delays or transient
///    errors. The sweep does a HEAD request: if the image is a 404, the capture is
///    marked failed; otherwise the metadata worker is notified to retry.
pub async fn run(
    ctx: ServiceContext,
    pool: PgPool,
    http: reqwest::Client,
    metadata_tx: tokio::sync::mpsc::UnboundedSender<String>,
) {
    let mut ticker = interval(Duration::from_secs(SWEEP_INTERVAL_SECS));

    while ctx.tick(&mut ticker).await {
        sweep_orphaned_uploads(&pool, &http).await;
        sweep_unverified_completions(&pool, &http, &metadata_tx).await;
    }
}

/// Check captures stuck in 'uploading' beyond the threshold.
/// HEAD request to R2 determines whether to complete or fail them.
async fn sweep_orphaned_uploads(pool: &PgPool, http: &reqwest::Client) {
    let captures =
        match CaptureRepo::list_stale_uploading(pool, UPLOADING_ORPHAN_THRESHOLD_SECS).await {
            Ok(c) => c,
            Err(e) => {
                error!(error = %e, "Integrity sweep: failed to list stale uploading captures");
                return;
            }
        };

    if captures.is_empty() {
        trace!("Integrity sweep: no orphaned uploads found");
        return;
    }

    info!(
        count = captures.len(),
        "Integrity sweep: checking orphaned uploads"
    );

    let mut completed = 0u32;
    let mut failed = 0u32;

    for capture in &captures {
        let id = capture.id.as_ref();

        let image_url = match capture.image_url {
            Some(ref url) if !url.is_empty() => url.as_str(),
            _ => {
                // No URL at all — definitely failed
                if let Err(e) =
                    CaptureRepo::mark_failed(pool, id, "Orphaned upload: no image URL was set")
                        .await
                {
                    error!(capture_id = id, error = %e, "Failed to mark orphaned capture as failed");
                }
                failed += 1;
                continue;
            }
        };

        match head_check(http, image_url).await {
            HeadResult::Exists => {
                // Image is in R2 but confirmation was lost — complete it
                match CaptureRepo::confirm_upload(pool, id, None).await {
                    Ok(true) => {
                        info!(
                            capture_id = id,
                            "Integrity sweep: recovered orphaned upload → completed"
                        );
                        completed += 1;
                    }
                    Ok(false) => {
                        debug!(
                            capture_id = id,
                            "Integrity sweep: capture already transitioned"
                        );
                    }
                    Err(e) => {
                        error!(capture_id = id, error = %e, "Integrity sweep: failed to confirm orphaned upload");
                    }
                }
            }
            HeadResult::NotFound => {
                if let Err(e) = CaptureRepo::mark_failed(
                    pool,
                    id,
                    "Orphaned upload: image not found in R2 after upload timeout",
                )
                .await
                {
                    error!(capture_id = id, error = %e, "Failed to mark orphaned capture as failed");
                }
                failed += 1;
            }
            HeadResult::Error(e) => {
                warn!(
                    capture_id = id,
                    error = %e,
                    "Integrity sweep: HEAD request failed for orphaned upload, will retry next sweep"
                );
            }
        }
    }

    if completed > 0 || failed > 0 {
        info!(
            completed,
            failed,
            total = captures.len(),
            "Integrity sweep: orphaned upload resolution complete"
        );
    }
}

/// Check completed captures that are still missing metadata after the grace period.
/// HEAD request to R2 determines whether to re-queue for metadata or mark failed.
async fn sweep_unverified_completions(
    pool: &PgPool,
    http: &reqwest::Client,
    metadata_tx: &tokio::sync::mpsc::UnboundedSender<String>,
) {
    let captures =
        match CaptureRepo::list_unverified_completed(pool, METADATA_GRACE_PERIOD_SECS).await {
            Ok(c) => c,
            Err(e) => {
                error!(error = %e, "Integrity sweep: failed to list unverified completions");
                return;
            }
        };

    if captures.is_empty() {
        trace!("Integrity sweep: no unverified completions found");
        return;
    }

    info!(
        count = captures.len(),
        "Integrity sweep: checking unverified completions"
    );

    let mut requeued = 0u32;
    let mut failed = 0u32;

    for capture in &captures {
        let id = capture.id.as_ref();

        let image_url = match capture.image_url {
            Some(ref url) if !url.is_empty() => url.as_str(),
            _ => {
                if let Err(e) =
                    CaptureRepo::mark_failed(pool, id, "Completed capture has no image URL").await
                {
                    error!(capture_id = id, error = %e, "Failed to mark capture as failed");
                }
                failed += 1;
                continue;
            }
        };

        match head_check(http, image_url).await {
            HeadResult::Exists => {
                // Image exists but metadata processing failed — re-queue
                let _ = metadata_tx.send(id.to_string());
                requeued += 1;
                debug!(
                    capture_id = id,
                    "Integrity sweep: re-queued for metadata processing"
                );
            }
            HeadResult::NotFound => {
                if let Err(e) = CaptureRepo::mark_failed(
                    pool,
                    id,
                    "Image not found in R2 despite completed status",
                )
                .await
                {
                    error!(capture_id = id, error = %e, "Failed to mark capture as failed");
                }
                warn!(
                    capture_id = id,
                    "Integrity sweep: completed capture has missing image → failed"
                );
                failed += 1;
            }
            HeadResult::Error(e) => {
                warn!(
                    capture_id = id,
                    error = %e,
                    "Integrity sweep: HEAD request failed, will retry next sweep"
                );
            }
        }
    }

    if requeued > 0 || failed > 0 {
        info!(
            requeued,
            failed,
            total = captures.len(),
            "Integrity sweep: unverified completion resolution complete"
        );
    }
}

enum HeadResult {
    Exists,
    NotFound,
    Error(reqwest::Error),
}

async fn head_check(client: &reqwest::Client, url: &str) -> HeadResult {
    match client.head(url).send().await {
        Ok(resp) if resp.status().is_success() => HeadResult::Exists,
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => HeadResult::NotFound,
        // Treat other HTTP errors (403, 500, etc.) as transient — don't mark failed
        Ok(resp) => HeadResult::Error(resp.error_for_status().unwrap_err()),
        Err(e) => HeadResult::Error(e),
    }
}
