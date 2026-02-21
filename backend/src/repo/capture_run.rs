use anyhow::Context;
use chrono::{DateTime, Utc};
use tracing::{debug, instrument, warn};

use crate::db::DbPool;
use crate::error::{AppResult, OptionNotFoundExt};
use crate::graphql::types::connection::CursorPage;
use crate::id::{
    CaptureId, CaptureRunId, SceneId, ScenePresetId, ShaderVersionId, ShaderVersionProfileId,
};
use crate::models::{
    CaptureRun, CaptureRunItem, CaptureRunItemStatus, CaptureRunItemWithContext, CaptureRunStatus,
};

/// Filters for cursor-paginated capture run listing.
#[derive(Default)]
pub struct CaptureRunFilters {
    /// Filter by run status (OR logic).
    pub statuses: Option<Vec<CaptureRunStatus>>,
    /// Only runs started after this date.
    pub started_after: Option<DateTime<Utc>>,
    /// Only runs started before this date.
    pub started_before: Option<DateTime<Utc>>,
    /// Minimum duration in seconds (completed runs only).
    pub min_duration_secs: Option<i64>,
    /// Maximum duration in seconds (completed runs only).
    pub max_duration_secs: Option<i64>,
}

/// Tuple for batch-inserting run items:
/// `(id, run_id, shader_version_id, scene_id, profile_id, preset_id)`
type RunItemTuple<'a> = (
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
);

pub struct CaptureRunRepo;

impl CaptureRunRepo {
    /// Create a new capture run
    #[instrument(skip(executor), level = "debug")]
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        agent_id: Option<&str>,
        total_items: i32,
        resolution_width: i32,
        resolution_height: i32,
        image_format: &str,
        metadata_json: Option<&serde_json::Value>,
    ) -> AppResult<CaptureRun> {
        let run = sqlx::query_as!(
            CaptureRun,
            r#"
            INSERT INTO capture_runs (id, agent_id, total_items, resolution_width, resolution_height, image_format, metadata_json, status, started_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, 'running', now())
            RETURNING id AS "id: CaptureRunId", agent_id, started_at, completed_at,
                status AS "status!: CaptureRunStatus",
                total_items,
                completed_items,
                failed_items,
                skipped_items,
                resolution_width,
                resolution_height,
                image_format,
                metadata_json as "metadata_json: serde_json::Value"
            "#,
            id,
            agent_id,
            total_items,
            resolution_width,
            resolution_height,
            image_format,
            metadata_json,
        )
        .fetch_one(executor)
        .await
        .context("failed to create capture run")?;

        debug!(run_id = id, "Created capture run");
        Ok(run)
    }

    /// Get a capture run by ID
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<CaptureRun> {
        sqlx::query_as!(
            CaptureRun,
            r#"
            SELECT
                cr.id AS "id: CaptureRunId", cr.agent_id, cr.started_at, cr.completed_at,
                cr.status AS "status!: CaptureRunStatus", cr.total_items,
                COALESCE(counts.completed, 0)::int4 as "completed_items!",
                COALESCE(counts.failed, 0)::int4 as "failed_items!",
                COALESCE(counts.skipped, 0)::int4 as "skipped_items!",
                cr.resolution_width,
                cr.resolution_height,
                cr.image_format,
                cr.metadata_json as "metadata_json: serde_json::Value"
            FROM capture_runs cr
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) FILTER (WHERE status = 'completed') AS completed,
                    COUNT(*) FILTER (WHERE status = 'failed') AS failed,
                    COUNT(*) FILTER (WHERE status = 'skipped') AS skipped
                FROM capture_run_items WHERE run_id = cr.id
            ) counts ON TRUE
            WHERE cr.id = $1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find capture run '{}'", id))?
        .or_not_found("Capture run", id)
    }

    /// List all capture runs (admin)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<CaptureRun>> {
        let runs = sqlx::query_as!(
            CaptureRun,
            r#"
            SELECT
                cr.id AS "id: CaptureRunId", cr.agent_id, cr.started_at, cr.completed_at,
                cr.status AS "status!: CaptureRunStatus", cr.total_items,
                COALESCE(counts.completed, 0)::int4 as "completed_items!",
                COALESCE(counts.failed, 0)::int4 as "failed_items!",
                COALESCE(counts.skipped, 0)::int4 as "skipped_items!",
                cr.resolution_width,
                cr.resolution_height,
                cr.image_format,
                cr.metadata_json as "metadata_json: serde_json::Value"
            FROM capture_runs cr
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) FILTER (WHERE status = 'completed') AS completed,
                    COUNT(*) FILTER (WHERE status = 'failed') AS failed,
                    COUNT(*) FILTER (WHERE status = 'skipped') AS skipped
                FROM capture_run_items WHERE run_id = cr.id
            ) counts ON TRUE
            ORDER BY cr.started_at DESC
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to list capture runs")?;

        Ok(runs)
    }

    /// Cursor-paginated list of capture runs with filtering.
    /// Default order: `started_at DESC, id DESC`. Keyset cursor on `(started_at, id)`.
    ///
    /// Supported sort values (prefix `-` for descending):
    /// `startedAt` (default DESC), `durationSecs`, `status`, `totalItems`.
    #[instrument(skip(db, filters), level = "debug")]
    pub async fn list_cursor(
        db: &DbPool,
        limit: i32,
        after: Option<(DateTime<Utc>, String)>,
        filters: &CaptureRunFilters,
        sort: Option<&str>,
    ) -> AppResult<CursorPage<CaptureRun>> {
        use sqlx::QueryBuilder;

        let limit = limit.clamp(1, 100) as i64;

        // Base SELECT with the same LATERAL join pattern as `list`
        let base_from = r#"
            FROM capture_runs cr
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) FILTER (WHERE status = 'completed') AS completed,
                    COUNT(*) FILTER (WHERE status = 'failed') AS failed,
                    COUNT(*) FILTER (WHERE status = 'skipped') AS skipped
                FROM capture_run_items WHERE run_id = cr.id
            ) counts ON TRUE"#;

        let select_cols = r#"
            cr.id,
            cr.agent_id,
            cr.started_at,
            cr.completed_at,
            cr.status,
            cr.total_items,
            COALESCE(counts.completed, 0)::int4 AS completed_items,
            COALESCE(counts.failed, 0)::int4 AS failed_items,
            COALESCE(counts.skipped, 0)::int4 AS skipped_items,
            cr.resolution_width,
            cr.resolution_height,
            cr.image_format,
            cr.metadata_json"#;

        // Helper closure to push shared WHERE filters
        let push_filters = |qb: &mut QueryBuilder<'_, sqlx::Postgres>| {
            if let Some(ref statuses) = filters.statuses
                && !statuses.is_empty()
            {
                qb.push(" AND cr.status IN (");
                let mut sep = qb.separated(", ");
                for s in statuses {
                    sep.push_bind(s.as_str().to_owned());
                }
                sep.push_unseparated(")");
            }
            if let Some(started_after) = filters.started_after {
                qb.push(" AND cr.started_at >= ");
                qb.push_bind(started_after);
            }
            if let Some(started_before) = filters.started_before {
                qb.push(" AND cr.started_at <= ");
                qb.push_bind(started_before);
            }
            if let Some(min_secs) = filters.min_duration_secs {
                qb.push(
                    " AND cr.completed_at IS NOT NULL AND EXTRACT(EPOCH FROM (cr.completed_at - cr.started_at)) >= ",
                );
                qb.push_bind(min_secs as f64);
            }
            if let Some(max_secs) = filters.max_duration_secs {
                qb.push(
                    " AND cr.completed_at IS NOT NULL AND EXTRACT(EPOCH FROM (cr.completed_at - cr.started_at)) <= ",
                );
                qb.push_bind(max_secs as f64);
            }
        };

        // Main data query
        let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("SELECT ");
        qb.push(select_cols);
        qb.push(base_from);
        qb.push(" WHERE TRUE");
        push_filters(&mut qb);

        if let Some((ref cursor_ts, ref cursor_id)) = after {
            qb.push(" AND (cr.started_at, cr.id) < (");
            qb.push_bind(*cursor_ts);
            qb.push(", ");
            qb.push_bind(cursor_id.as_str());
            qb.push(")");
        }

        let sort_value = sort.unwrap_or("-startedAt");
        match sort_value {
            "startedAt" => qb.push(" ORDER BY cr.started_at ASC, cr.id ASC"),
            "durationSecs" => qb.push(
                " ORDER BY EXTRACT(EPOCH FROM (cr.completed_at - cr.started_at)) ASC NULLS LAST, cr.id ASC",
            ),
            "-durationSecs" => qb.push(
                " ORDER BY EXTRACT(EPOCH FROM (cr.completed_at - cr.started_at)) DESC NULLS LAST, cr.id DESC",
            ),
            "status" => {
                qb.push(" ORDER BY cr.status ASC, cr.started_at DESC, cr.id DESC")
            }
            "-status" => {
                qb.push(" ORDER BY cr.status DESC, cr.started_at DESC, cr.id DESC")
            }
            "totalItems" => {
                qb.push(" ORDER BY cr.total_items ASC, cr.started_at DESC, cr.id DESC")
            }
            "-totalItems" => {
                qb.push(" ORDER BY cr.total_items DESC, cr.started_at DESC, cr.id DESC")
            }
            // Default: most recent first
            _ => qb.push(" ORDER BY cr.started_at DESC, cr.id DESC"),
        };

        qb.push(" LIMIT ");
        qb.push_bind(limit + 1);

        let items: Vec<CaptureRun> = qb
            .build_query_as()
            .fetch_all(db)
            .await
            .context("failed to list capture runs (cursor)")?;

        let has_next_page = items.len() as i64 > limit;
        let items: Vec<CaptureRun> = items.into_iter().take(limit as usize).collect();

        // Count query (same filters, no cursor/limit).
        // Intentionally omits the LATERAL join since current filters only reference
        // `cr.*` columns. If a future filter needs computed counts (e.g., "runs with
        // >5 failures"), the LATERAL join must be added here too.
        let mut cqb: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM capture_runs cr WHERE TRUE");
        push_filters(&mut cqb);

        let total_count: i64 = cqb
            .build_query_scalar::<Option<i64>>()
            .fetch_one(db)
            .await
            .context("failed to count capture runs")?
            .unwrap_or(0);

        debug!(total_count, has_next_page, "Listed capture runs (cursor)");
        Ok(CursorPage {
            items,
            has_next_page,
            has_previous_page: after.is_some(),
            total_count,
        })
    }

    /// Complete a capture run (update counters and status)
    #[instrument(skip(executor), level = "debug")]
    pub async fn complete(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<CaptureRun> {
        let run = sqlx::query_as!(
            CaptureRun,
            r#"
            WITH counts AS (
                SELECT
                    COUNT(*) FILTER (WHERE status = 'completed') AS completed,
                    COUNT(*) FILTER (WHERE status = 'failed') AS failed,
                    COUNT(*) FILTER (WHERE status = 'skipped') AS skipped
                FROM capture_run_items WHERE run_id = $1
            )
            UPDATE capture_runs SET
                completed_at = now(),
                completed_items = counts.completed::int4,
                failed_items = counts.failed::int4,
                skipped_items = counts.skipped::int4,
                status = CASE
                    WHEN counts.failed > 0 AND counts.completed > 0 THEN 'partial'
                    WHEN counts.completed = 0 THEN 'failed'
                    ELSE 'completed'
                END
            FROM counts
            WHERE capture_runs.id = $1
            RETURNING capture_runs.id AS "id: CaptureRunId", capture_runs.agent_id,
                capture_runs.started_at, capture_runs.completed_at,
                capture_runs.status AS "status!: CaptureRunStatus",
                capture_runs.total_items,
                capture_runs.completed_items,
                capture_runs.failed_items,
                capture_runs.skipped_items,
                capture_runs.resolution_width,
                capture_runs.resolution_height,
                capture_runs.image_format,
                capture_runs.metadata_json as "metadata_json: serde_json::Value"
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to complete capture run '{}'", id))?
        .or_not_found("Capture run", id)?;

        debug!(run_id = id, status = %run.status, "Completed capture run");
        Ok(run)
    }

    /// Insert a batch of run items in a single query
    #[instrument(skip(executor, items), level = "debug")]
    pub async fn insert_items(
        executor: impl sqlx::PgExecutor<'_>,
        items: &[RunItemTuple<'_>],
    ) -> AppResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let ids: Vec<&str> = items.iter().map(|i| i.0).collect();
        let run_ids: Vec<&str> = items.iter().map(|i| i.1).collect();
        let sv_ids: Vec<&str> = items.iter().map(|i| i.2).collect();
        let scene_ids: Vec<&str> = items.iter().map(|i| i.3).collect();
        let profiles: Vec<Option<&str>> = items.iter().map(|i| i.4).collect();
        let presets: Vec<Option<&str>> = items.iter().map(|i| i.5).collect();

        sqlx::query!(
            r#"
            INSERT INTO capture_run_items (id, run_id, shader_version_id, scene_id, profile_id, preset_id, status)
            SELECT unnest($1::text[]), unnest($2::text[]), unnest($3::text[]), unnest($4::text[]), unnest($5::text[]), unnest($6::text[]), 'pending'
            "#,
            &ids as &[&str],
            &run_ids as &[&str],
            &sv_ids as &[&str],
            &scene_ids as &[&str],
            &profiles as &[Option<&str>],
            &presets as &[Option<&str>],
        )
        .execute(executor)
        .await
        .context("failed to insert capture run items")?;

        debug!(count = items.len(), "Inserted capture run items");
        Ok(())
    }

    /// Mark a run item as completed with a linked capture
    #[instrument(skip(executor), level = "debug")]
    pub async fn complete_item(
        executor: impl sqlx::PgExecutor<'_>,
        item_id: &str,
        capture_id: &str,
        duration_ms: Option<i32>,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE capture_run_items
            SET status = 'completed', capture_id = $2, duration_ms = $3, completed_at = now()
            WHERE id = $1 AND status IN ('pending', 'running')
            "#,
            item_id,
            capture_id,
            duration_ms,
        )
        .execute(executor)
        .await
        .context(format!("failed to complete run item '{}'", item_id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark a run item as failed
    #[instrument(skip(executor), level = "debug")]
    pub async fn fail_item(
        executor: impl sqlx::PgExecutor<'_>,
        item_id: &str,
        error_message: &str,
        error_log: Option<&str>,
        duration_ms: Option<i32>,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE capture_run_items
            SET status = 'failed', error_message = $2, error_log = $3, duration_ms = $4, completed_at = now()
            WHERE id = $1 AND status IN ('pending', 'running')
            "#,
            item_id,
            error_message,
            error_log,
            duration_ms,
        )
        .execute(executor)
        .await
        .context(format!("failed to fail run item '{}'", item_id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Get a single run item by ID
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_item_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        item_id: &str,
    ) -> AppResult<CaptureRunItem> {
        sqlx::query_as!(
            CaptureRunItem,
            r#"SELECT id,
                run_id AS "run_id: CaptureRunId",
                shader_version_id AS "shader_version_id: ShaderVersionId",
                scene_id AS "scene_id: SceneId",
                profile_id AS "profile_id: ShaderVersionProfileId",
                preset_id AS "preset_id: ScenePresetId",
                status AS "status!: CaptureRunItemStatus",
                capture_id AS "capture_id: CaptureId",
                error_message, error_log, duration_ms,
                started_at, completed_at
            FROM capture_run_items WHERE id = $1"#,
            item_id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find run item '{}'", item_id))?
        .or_not_found("Run item", item_id)
    }

    /// Claim a run item: set status to 'running' and link a capture
    #[instrument(skip(executor), level = "debug")]
    pub async fn claim_item(
        executor: impl sqlx::PgExecutor<'_>,
        item_id: &str,
        capture_id: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "UPDATE capture_run_items SET status = 'running', capture_id = $2, started_at = now() WHERE id = $1",
            item_id,
            capture_id,
        )
        .execute(executor)
        .await
        .context(format!("failed to claim run item '{}'", item_id))?;
        Ok(())
    }

    /// List items for a capture run
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_items(
        executor: impl sqlx::PgExecutor<'_>,
        run_id: &str,
    ) -> AppResult<Vec<CaptureRunItem>> {
        let items = sqlx::query_as!(
            CaptureRunItem,
            r#"SELECT id,
                run_id AS "run_id: CaptureRunId",
                shader_version_id AS "shader_version_id: ShaderVersionId",
                scene_id AS "scene_id: SceneId",
                profile_id AS "profile_id: ShaderVersionProfileId",
                preset_id AS "preset_id: ScenePresetId",
                status AS "status!: CaptureRunItemStatus",
                capture_id AS "capture_id: CaptureId",
                error_message, error_log, duration_ms,
                started_at, completed_at
            FROM capture_run_items WHERE run_id = $1 ORDER BY started_at ASC NULLS LAST"#,
            run_id
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list items for run '{}'", run_id))?;

        Ok(items)
    }

    /// List items for a capture run with shader/scene context
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_items_with_context(
        executor: impl sqlx::PgExecutor<'_>,
        run_id: &str,
    ) -> AppResult<Vec<CaptureRunItemWithContext>> {
        let items = sqlx::query_as!(
            CaptureRunItemWithContext,
            r#"
            SELECT
                cri.id,
                cri.run_id AS "run_id: CaptureRunId",
                cri.shader_version_id AS "shader_version_id: ShaderVersionId",
                cri.scene_id AS "scene_id: SceneId",
                cri.profile_id AS "profile_id: ShaderVersionProfileId",
                svp.name AS "profile_name?",
                svp.display_name AS "profile_display_name?",
                cri.preset_id AS "preset_id: ScenePresetId",
                cri.status AS "status!: CaptureRunItemStatus",
                cri.capture_id AS "capture_id: CaptureId",
                cri.error_message, cri.error_log, cri.duration_ms,
                cri.started_at, cri.completed_at,
                s.name as shader_name,
                s.slug as shader_slug,
                sv.version as shader_version,
                sc.name as scene_name,
                c.image_path as "image_path?",
                c.thumbhash as "thumbhash?"
            FROM capture_run_items cri
            JOIN shader_versions sv ON cri.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            JOIN scenes sc ON cri.scene_id = sc.id
            LEFT JOIN shader_version_profiles svp ON cri.profile_id = svp.id
            LEFT JOIN captures c ON cri.capture_id = c.id
            WHERE cri.run_id = $1
            ORDER BY cri.started_at ASC NULLS LAST
            "#,
            run_id
        )
        .fetch_all(executor)
        .await
        .context(format!(
            "failed to list items with context for run '{}'",
            run_id
        ))?;

        Ok(items)
    }

    /// Find capture runs that are still 'running' but have had no item activity
    /// within the given timeout period. "Activity" is the most recent item
    /// `started_at` or `completed_at`, falling back to the run's `started_at`.
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_stale_runs(
        executor: impl sqlx::PgExecutor<'_>,
        timeout_secs: i64,
    ) -> AppResult<Vec<String>> {
        let rows = sqlx::query_scalar!(
            r#"
            SELECT cr.id as "id!"
            FROM capture_runs cr
            LEFT JOIN LATERAL (
                SELECT GREATEST(
                    MAX(cri.started_at),
                    MAX(cri.completed_at)
                ) AS last_activity
                FROM capture_run_items cri
                WHERE cri.run_id = cr.id
            ) activity ON TRUE
            WHERE cr.status = 'running'
              AND COALESCE(activity.last_activity, cr.started_at)
                  < now() - make_interval(secs => $1::float8)
            "#,
            timeout_secs as f64
        )
        .fetch_all(executor)
        .await
        .context("failed to find stale capture runs")?;

        Ok(rows)
    }

    /// Mark a run as timed out: set remaining pending/running items to 'skipped',
    /// tally final counts, and set the run status to 'timed_out'.
    #[instrument(skip(pool), level = "debug")]
    pub async fn timeout_run(pool: &sqlx::PgPool, run_id: &str) -> AppResult<()> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        // Skip any items still pending or running
        let skipped = sqlx::query!(
            r#"
            UPDATE capture_run_items
            SET status = 'skipped', completed_at = now()
            WHERE run_id = $1 AND status IN ('pending', 'running')
            "#,
            run_id
        )
        .execute(&mut *tx)
        .await
        .context("failed to skip remaining items")?;

        // Finalize the run with accurate counts.
        // Set completed_at to the last real item activity (claim/complete/fail),
        // not now(), so the run's duration reflects actual work rather than
        // including the idle timeout period.
        sqlx::query!(
            r#"
            WITH counts AS (
                SELECT
                    COUNT(*) FILTER (WHERE status = 'completed') AS completed,
                    COUNT(*) FILTER (WHERE status = 'failed') AS failed,
                    COUNT(*) FILTER (WHERE status = 'skipped') AS skipped
                FROM capture_run_items WHERE run_id = $1
            ),
            last_activity AS (
                SELECT GREATEST(MAX(started_at), MAX(completed_at)) AS ts
                FROM capture_run_items
                WHERE run_id = $1
            )
            UPDATE capture_runs SET
                completed_at = COALESCE(last_activity.ts, capture_runs.started_at),
                completed_items = counts.completed::int4,
                failed_items = counts.failed::int4,
                skipped_items = counts.skipped::int4,
                status = 'timed_out'
            FROM counts, last_activity
            WHERE capture_runs.id = $1
            "#,
            run_id
        )
        .execute(&mut *tx)
        .await
        .context("failed to mark run as timed out")?;

        tx.commit()
            .await
            .context("failed to commit timeout transaction")?;

        warn!(
            run_id,
            items_skipped = skipped.rows_affected(),
            "Capture run timed out"
        );
        Ok(())
    }
}
