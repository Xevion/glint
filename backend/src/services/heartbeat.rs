use std::time::Duration;

use sqlx::SqlitePool;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::config::HeartbeatConfig;

/// Background task that monitors job heartbeats and resets stale jobs.
/// Uses adaptive polling: faster when jobs are active, slower when idle.
pub async fn monitor_heartbeats(pool: SqlitePool, config: HeartbeatConfig) {
    info!(
        "Starting heartbeat monitor (timeout: {}s, active poll: {}s, idle poll: {}s)",
        config.timeout_seconds, config.active_poll_seconds, config.idle_poll_seconds
    );

    let mut current_interval = Duration::from_secs(config.idle_poll_seconds);
    let mut ticker = interval(current_interval);

    loop {
        ticker.tick().await;

        match reset_stale_jobs(&pool, config.timeout_seconds).await {
            Ok(reset_count) => {
                if reset_count > 0 {
                    warn!("Reset {} stale job(s)", reset_count);
                }

                // Check if there are active jobs to determine next interval
                let has_active_jobs = match check_active_jobs(&pool).await {
                    Ok(active) => active,
                    Err(e) => {
                        error!("Failed to check for active jobs: {}", e);
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
                        "Adjusting heartbeat poll interval: {:?} -> {:?} (active_jobs: {})",
                        current_interval, new_interval, has_active_jobs
                    );
                    current_interval = new_interval;
                    ticker = interval(new_interval);
                }
            }
            Err(e) => {
                error!("Failed to reset stale jobs: {}", e);
            }
        }
    }
}

/// Reset jobs that have exceeded the heartbeat timeout
async fn reset_stale_jobs(pool: &SqlitePool, timeout_seconds: u64) -> anyhow::Result<u64> {
    let timeout_str = format!("-{} seconds", timeout_seconds);

    let result = sqlx::query(
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
          AND last_heartbeat < datetime('now', 'utc', ?)
        "#,
    )
    .bind(&timeout_str)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Check if there are any active (claimed or running) jobs
async fn check_active_jobs(pool: &SqlitePool) -> anyhow::Result<bool> {
    let result: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM jobs WHERE status IN ('claimed', 'running')")
            .fetch_one(pool)
            .await?;

    Ok(result.0 > 0)
}
