use anyhow::Context;
use tracing::instrument;

use crate::error::AppResult;
use crate::models::stats::Stats;

pub struct StatsRepo;

impl StatsRepo {
    /// Aggregate counts for the public index page.
    #[instrument(skip(executor), level = "debug")]
    pub async fn get(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Stats> {
        let stats = sqlx::query_as!(
            Stats,
            r#"
            SELECT
                (SELECT COUNT(*) FROM shaders) AS "shader_count!",
                (SELECT COUNT(*) FROM scenes WHERE active = TRUE) AS "scene_count!",
                (SELECT COUNT(*) FROM captures WHERE status = 'completed') AS "capture_count!",
                (SELECT COUNT(*) FROM users) AS "user_count!",
                (SELECT COUNT(*) FROM capture_runs) AS "run_count!",
                (SELECT MAX(captured_at) FROM captures WHERE status = 'completed') AS latest_capture_at
            "#,
        )
        .fetch_one(executor)
        .await
        .context("failed to fetch stats")?;

        Ok(stats)
    }
}
