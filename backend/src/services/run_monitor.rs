use std::time::Duration;

use sqlx::PgPool;
use tokio::time::interval;
use tracing::{error, trace, warn};

use crate::repo::CaptureRunRepo;
use crate::services::lifecycle::ServiceContext;

/// How often the monitor checks for stale runs (60 seconds)
const CHECK_INTERVAL_SECS: u64 = 60;

/// A run is considered stale after 5 minutes of no item activity
const RUN_TIMEOUT_SECS: i64 = 5 * 60;

/// Background task that detects and times out stale capture runs.
///
/// A run is "stale" when it's still in 'running' status but has had no item
/// activity (claim, completion, or failure) for longer than `RUN_TIMEOUT_SECS`.
/// Stale runs are transitioned to 'timed_out' with partial progress preserved.
pub async fn monitor_capture_runs(ctx: ServiceContext, pool: PgPool) {
    let mut ticker = interval(Duration::from_secs(CHECK_INTERVAL_SECS));

    while ctx.tick(&mut ticker).await {
        match CaptureRunRepo::find_stale_runs(&pool, RUN_TIMEOUT_SECS).await {
            Ok(stale_ids) => {
                if stale_ids.is_empty() {
                    trace!("No stale capture runs found");
                    continue;
                }

                warn!(count = stale_ids.len(), "Found stale capture runs");

                for run_id in &stale_ids {
                    if let Err(e) = CaptureRunRepo::timeout_run(&pool, run_id).await {
                        error!(run_id, error = %e, "Failed to timeout capture run");
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to check for stale capture runs");
            }
        }
    }
}
