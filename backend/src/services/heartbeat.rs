use std::time::Duration;

use sqlx::PgPool;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::config::HeartbeatConfig;

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

        match reset_stale_jobs(&pool, config.timeout_seconds).await {
            Ok(reset_count) => {
                if reset_count > 0 {
                    warn!(count = reset_count, "Reset stale jobs");
                }

                // Check if there are active jobs to determine next interval
                let has_active_jobs = match check_active_jobs(&pool).await {
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

/// Reset jobs that have exceeded the heartbeat timeout
async fn reset_stale_jobs(pool: &PgPool, timeout_seconds: u64) -> anyhow::Result<u64> {
    let timeout_secs = timeout_seconds as f64;
    let result = sqlx::query!(
        r#"
        UPDATE jobs
        SET status = CASE
            WHEN attempts >= max_attempts THEN 'failed'
            ELSE 'pending'
        END,
        error_message = CASE
            WHEN attempts >= max_attempts THEN 'Heartbeat timeout after ' || attempts || ' attempts'
            ELSE NULL
        END,
        agent_id = NULL,
        claimed_at = NULL,
        last_heartbeat = NULL
        WHERE status IN ('claimed', 'running')
          AND last_heartbeat < now() - make_interval(secs => $1)
        "#,
        timeout_secs
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Check if there are any active (claimed or running) jobs
async fn check_active_jobs(pool: &PgPool) -> anyhow::Result<bool> {
    let result = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::int8 as "count!" FROM jobs WHERE status IN ('claimed', 'running')"#
    )
    .fetch_one(pool)
    .await?;

    Ok(result > 0)
}
