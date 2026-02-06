use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::{AppError, AppResult};
use crate::models::{CaptureRun, CaptureRunItem, CaptureRunItemWithContext};

pub struct CaptureRunRepo;

impl CaptureRunRepo {
    /// Create a new capture run
    #[instrument(skip(executor), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        agent_id: Option<&str>,
        total_items: i32,
        metadata_json: Option<&str>,
    ) -> AppResult<CaptureRun> {
        let run = sqlx::query_as!(
            CaptureRun,
            r#"
            INSERT INTO capture_runs (id, agent_id, total_items, metadata_json, status, started_at)
            VALUES ($1, $2, $3, $4, 'running', now())
            RETURNING *
            "#,
            id,
            agent_id,
            total_items,
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
                cr.id, cr.agent_id, cr.started_at, cr.completed_at,
                cr.status, cr.total_items,
                COALESCE(counts.completed, 0)::int4 as "completed_items!",
                COALESCE(counts.failed, 0)::int4 as "failed_items!",
                COALESCE(counts.skipped, 0)::int4 as "skipped_items!",
                cr.metadata_json
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
        .ok_or_else(|| AppError::NotFound(format!("Capture run '{}' not found", id)))
    }

    /// List all capture runs (admin)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<CaptureRun>> {
        let runs = sqlx::query_as!(
            CaptureRun,
            r#"
            SELECT
                cr.id, cr.agent_id, cr.started_at, cr.completed_at,
                cr.status, cr.total_items,
                COALESCE(counts.completed, 0)::int4 as "completed_items!",
                COALESCE(counts.failed, 0)::int4 as "failed_items!",
                COALESCE(counts.skipped, 0)::int4 as "skipped_items!",
                cr.metadata_json
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
            RETURNING capture_runs.*
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to complete capture run '{}'", id))?
        .ok_or_else(|| AppError::NotFound(format!("Capture run '{}' not found", id)))?;

        debug!(run_id = id, status = %run.status, "Completed capture run");
        Ok(run)
    }

    /// Insert a batch of run items in a single query
    #[instrument(skip(executor, items), level = "debug")]
    pub async fn insert_items(
        executor: impl sqlx::PgExecutor<'_>,
        items: &[(&str, &str, &str, &str, Option<&str>)], // (id, run_id, shader_version_id, scene_id, profile)
    ) -> AppResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let ids: Vec<&str> = items.iter().map(|i| i.0).collect();
        let run_ids: Vec<&str> = items.iter().map(|i| i.1).collect();
        let sv_ids: Vec<&str> = items.iter().map(|i| i.2).collect();
        let scene_ids: Vec<&str> = items.iter().map(|i| i.3).collect();
        let profiles: Vec<Option<&str>> = items.iter().map(|i| i.4).collect();

        sqlx::query!(
            r#"
            INSERT INTO capture_run_items (id, run_id, shader_version_id, scene_id, profile, status)
            SELECT unnest($1::text[]), unnest($2::text[]), unnest($3::text[]), unnest($4::text[]), unnest($5::text[]), 'pending'
            "#,
            &ids as &[&str],
            &run_ids as &[&str],
            &sv_ids as &[&str],
            &scene_ids as &[&str],
            &profiles as &[Option<&str>],
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
            "SELECT * FROM capture_run_items WHERE id = $1",
            item_id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find run item '{}'", item_id))?
        .ok_or_else(|| AppError::NotFound(format!("Run item '{}' not found", item_id)))
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
            "SELECT * FROM capture_run_items WHERE run_id = $1 ORDER BY started_at ASC NULLS LAST",
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
                cri.id, cri.run_id, cri.shader_version_id, cri.scene_id,
                cri.profile, cri.status, cri.capture_id,
                cri.error_message, cri.error_log, cri.duration_ms,
                cri.started_at, cri.completed_at,
                s.name as shader_name,
                s.slug as shader_slug,
                sv.version as shader_version,
                sc.name as scene_name
            FROM capture_run_items cri
            JOIN shader_versions sv ON cri.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            JOIN scenes sc ON cri.scene_id = sc.id
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
}
