use std::time::Duration;

use sqlx::PgPool;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::config::HeartbeatConfig;
use crate::repo::JobRepo;

/// Background task that monitors job heartbeats and resets stale jobs.
/// Uses adaptive polling: faster when jobs are active, slower when idle.
pub async fn monitor_heartbeats(pool: PgPool, config: HeartbeatConfig) {
    info!(
        timeout_secs = config.timeout_seconds,
        active_poll_secs = config.active_poll_seconds,
        idle_poll_secs = config.idle_poll_seconds,
        "Heartbeat monitor started"
    );

    let mut current_interval = Duration::from_secs(config.idle_poll_seconds);
    let mut ticker = interval(current_interval);

    loop {
        ticker.tick().await;

        match JobRepo::reset_stale(&pool, config.timeout_seconds).await {
            Ok(reset_count) => {
                if reset_count > 0 {
                    warn!(count = reset_count, "Reset stale jobs");
                }

                // Check if there are active jobs to determine next interval
                let has_active_jobs = match JobRepo::has_active_jobs(&pool).await {
                    Ok(active) => active,
                    Err(e) => {
                        error!(error = %e, "Failed to check for active jobs");
                        false
                    }
                };

                let new_interval = if has_active_jobs {
                    Duration::from_secs(config.active_poll_seconds)
                } else {
                    Duration::from_secs(config.idle_poll_seconds)
                };

                if new_interval != current_interval {
                    debug!(
                        from_secs = current_interval.as_secs(),
                        to_secs = new_interval.as_secs(),
                        active_jobs = has_active_jobs,
                        "Poll interval adjusted"
                    );
                    current_interval = new_interval;
                    ticker = interval(new_interval);
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to reset stale jobs");
            }
        }
    }
}
