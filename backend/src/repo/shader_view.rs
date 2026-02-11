use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::AppResult;
use crate::id::ShaderId;

/// Trending shader entry from the repo layer: shader ID + view count in the time window.
pub struct TrendingEntry {
    pub shader_id: ShaderId,
    pub view_count: i64,
}

pub struct ShaderViewRepo;

impl ShaderViewRepo {
    /// Record a view, deduplicated per viewer per hour via the unique index.
    /// Returns true if a new row was inserted, false if it was a duplicate.
    #[instrument(skip(executor), level = "debug")]
    pub async fn record_view(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
        viewer_hash: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            INSERT INTO shader_views (shader_id, viewer_hash)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            shader_id,
            viewer_hash,
        )
        .execute(executor)
        .await
        .context("failed to record shader view")?;

        let inserted = result.rows_affected() > 0;
        debug!(shader_id, viewer_hash, inserted, "Recorded shader view");
        Ok(inserted)
    }

    /// Get top trending shaders by view count in the last N days.
    #[instrument(skip(executor), level = "debug")]
    pub async fn trending(
        executor: impl sqlx::PgExecutor<'_>,
        days: i32,
        limit: i64,
    ) -> AppResult<Vec<TrendingEntry>> {
        struct Row {
            shader_id: String,
            view_count: Option<i64>,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT shader_id, COUNT(*) as view_count
            FROM shader_views
            WHERE viewed_at > (now() AT TIME ZONE 'UTC') - make_interval(days => $1)
            GROUP BY shader_id
            ORDER BY view_count DESC
            LIMIT $2
            "#,
            days,
            limit,
        )
        .fetch_all(executor)
        .await
        .context("failed to query trending shaders")?;

        let trending = rows
            .into_iter()
            .map(|r| TrendingEntry {
                shader_id: ShaderId(r.shader_id),
                view_count: r.view_count.unwrap_or(0),
            })
            .collect();

        Ok(trending)
    }
}
