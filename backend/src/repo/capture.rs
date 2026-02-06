use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{Capture, CaptureWithContext};

pub struct CaptureRepo;

const CAPTURE_WITH_CONTEXT_BASE: &str = r#"
    SELECT
        c.id,
        c.scene_id,
        s.slug as shader_slug,
        s.name as shader_name,
        sv.version as shader_version,
        c.profile,
        c.image_path,
        c.image_url,
        c.captured_at,
        c.resolution_width,
        c.resolution_height,
        cri.run_id,
        cr.status as run_status,
        (SELECT sa.name FROM shader_authors sa WHERE sa.shader_id = s.id LIMIT 1) as shader_author
    FROM captures c
    JOIN shader_versions sv ON c.shader_version_id = sv.id
    JOIN shaders s ON sv.shader_id = s.id
    LEFT JOIN capture_run_items cri ON cri.capture_id = c.id
    LEFT JOIN capture_runs cr ON cri.run_id = cr.id
"#;

impl CaptureRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<Capture>> {
        sqlx::query_as!(Capture, "SELECT * FROM captures WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find capture '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<Capture> {
        Self::find_by_id(executor, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Capture '{}' not found", id)))
    }

    /// List all completed captures
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_completed(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<Capture>> {
        let captures = sqlx::query_as!(
            Capture,
            "SELECT * FROM captures WHERE status = 'completed' ORDER BY created_at DESC"
        )
        .fetch_all(executor)
        .await
        .context("failed to list completed captures")?;

        debug!(count = captures.len(), "Listed completed captures");
        Ok(captures)
    }

    /// List captures by shader version
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_shader_version(
        executor: impl sqlx::PgExecutor<'_>,
        shader_version_id: &str,
    ) -> AppResult<Vec<Capture>> {
        let captures = sqlx::query_as!(
            Capture,
            "SELECT * FROM captures WHERE shader_version_id = $1 ORDER BY created_at DESC",
            shader_version_id
        )
        .fetch_all(executor)
        .await
        .context(format!(
            "failed to list captures for shader version '{}'",
            shader_version_id
        ))?;

        debug!(count = captures.len(), "Listed captures for shader version");
        Ok(captures)
    }

    /// List captures by scene
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_scene(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<Vec<Capture>> {
        let captures = sqlx::query_as!(
            Capture,
            "SELECT * FROM captures WHERE scene_id = $1 ORDER BY created_at DESC",
            scene_id
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list captures for scene '{}'", scene_id))?;

        debug!(count = captures.len(), "Listed captures for scene");
        Ok(captures)
    }

    /// Fetch captures with shader/version context for a scene (for scene detail page)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_with_context_for_scene(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<Vec<CaptureWithContext>> {
        let sql = format!(
            "{} WHERE c.scene_id = $1 AND c.status = 'completed' ORDER BY s.name, sv.created_at DESC",
            CAPTURE_WITH_CONTEXT_BASE
        );
        let captures = sqlx::query_as::<_, CaptureWithContext>(&sql)
            .bind(scene_id)
            .fetch_all(executor)
            .await
            .context(format!(
                "failed to get captures with context for scene '{}'",
                scene_id
            ))?;

        debug!(count = captures.len(), "Fetched captures with context");
        Ok(captures)
    }

    /// Insert a new capture (append-only — no upsert)
    #[instrument(skip(executor), level = "debug")]
    #[allow(clippy::too_many_arguments)]
    pub async fn insert(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        shader_version_id: &str,
        scene_id: &str,
        profile: Option<&str>,
        image_path: Option<&str>,
        image_url: Option<&str>,
        resolution_width: Option<i32>,
        resolution_height: Option<i32>,
        captured_at: Option<DateTime<Utc>>,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO captures (
                id, shader_version_id, scene_id, profile, image_path, image_url,
                resolution_width, resolution_height, outdated, status, created_at, updated_at, captured_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, FALSE, 'completed', $9, $9, $9)
            "#,
            id,
            shader_version_id,
            scene_id,
            profile,
            image_path,
            image_url,
            resolution_width,
            resolution_height,
            captured_at
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to insert capture for scene '{}' with shader version '{}'",
            scene_id, shader_version_id
        ))?;

        debug!(scene_id, shader_version_id, "Capture inserted");
        Ok(())
    }

    /// Insert a capture in 'uploading' status (upload in flight, not yet confirmed)
    #[instrument(skip(executor), level = "debug")]
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_uploading(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        shader_version_id: &str,
        scene_id: &str,
        profile: Option<&str>,
        image_url: Option<&str>,
        resolution_width: Option<i32>,
        resolution_height: Option<i32>,
        captured_at: Option<DateTime<Utc>>,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO captures (
                id, shader_version_id, scene_id, profile, image_url,
                resolution_width, resolution_height, outdated, status, created_at, updated_at, captured_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, 'uploading', $8, $8, $8)
            "#,
            id,
            shader_version_id,
            scene_id,
            profile,
            image_url,
            resolution_width,
            resolution_height,
            captured_at
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to insert uploading capture for scene '{}' with shader version '{}'",
            scene_id, shader_version_id
        ))?;

        debug!(scene_id, shader_version_id, "Uploading capture inserted");
        Ok(())
    }

    /// Confirm an upload: transition capture from 'uploading' to 'completed'
    #[instrument(skip(executor), level = "debug")]
    pub async fn confirm_upload(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        image_path: Option<&str>,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE captures
            SET status = 'completed', image_path = $2, updated_at = now()
            WHERE id = $1 AND status = 'uploading'
            "#,
            id,
            image_path
        )
        .execute(executor)
        .await
        .context(format!("failed to confirm upload for capture '{}'", id))?;

        debug!(id, "Capture upload confirmed");
        Ok(result.rows_affected() > 0)
    }

    /// Mark captures as outdated for a scene (when scene is updated)
    #[instrument(skip(executor), level = "debug")]
    pub async fn mark_outdated_for_scene(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<u64> {
        let result = sqlx::query!(
            "UPDATE captures SET outdated = TRUE WHERE scene_id = $1 AND status = 'completed'",
            scene_id
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to mark captures outdated for scene '{}'",
            scene_id
        ))?;

        debug!(
            scene_id,
            count = result.rows_affected(),
            "Marked captures outdated"
        );
        Ok(result.rows_affected())
    }

    /// Count captures by status for dashboard stats
    #[instrument(skip(executor), level = "debug")]
    pub async fn count_by_status(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<CaptureStatusCounts> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending')::int4 as "pending!",
                COUNT(*) FILTER (WHERE status = 'completed')::int4 as "completed!",
                COUNT(*) FILTER (WHERE status = 'failed')::int4 as "failed!",
                COUNT(*) FILTER (WHERE outdated = TRUE)::int4 as "outdated!"
            FROM captures
            "#
        )
        .fetch_one(executor)
        .await
        .context("failed to count captures by status")?;

        Ok(CaptureStatusCounts {
            pending: row.pending,
            completed: row.completed,
            failed: row.failed,
            outdated: row.outdated,
        })
    }
}

#[derive(Debug)]
pub struct CaptureStatusCounts {
    pub pending: i32,
    pub completed: i32,
    pub failed: i32,
    pub outdated: i32,
}

impl CaptureRepo {
    /// List all captures with context (for admin dashboard)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_all_with_context(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<CaptureWithContext>> {
        let sql = format!("{} ORDER BY c.created_at DESC", CAPTURE_WITH_CONTEXT_BASE);
        let captures = sqlx::query_as::<_, CaptureWithContext>(&sql)
            .fetch_all(executor)
            .await
            .context("failed to list all captures with context")?;

        debug!(count = captures.len(), "Listed all captures with context");
        Ok(captures)
    }

    /// Get a single capture with context (for admin detail view)
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_with_context(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<CaptureWithContext> {
        let sql = format!("{} WHERE c.id = $1", CAPTURE_WITH_CONTEXT_BASE);
        sqlx::query_as::<_, CaptureWithContext>(&sql)
            .bind(id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to get capture with context '{}'", id))?
            .ok_or_else(|| AppError::NotFound(format!("Capture '{}' not found", id)))
    }

    /// List captures with context, pagination, and filtering (admin)
    #[instrument(skip(db), level = "debug")]
    pub async fn list_all_with_context_paginated(
        db: &DbPool,
        limit: i64,
        offset: i64,
        shader: Option<&str>,
        scene: Option<&str>,
        status: Option<&str>,
        run_id: Option<&str>,
    ) -> AppResult<(Vec<CaptureWithContext>, i64)> {
        let sql = format!(
            r#"{}
            WHERE ($1::text IS NULL OR s.slug = $1)
              AND ($2::text IS NULL OR c.scene_id = $2)
              AND ($3::text IS NULL OR c.status = $3)
              AND ($4::text IS NULL OR cri.run_id = $4)
            ORDER BY c.created_at DESC
            LIMIT $5 OFFSET $6
            "#,
            CAPTURE_WITH_CONTEXT_BASE
        );
        let items = sqlx::query_as::<_, CaptureWithContext>(&sql)
            .bind(shader)
            .bind(scene)
            .bind(status)
            .bind(run_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(db)
            .await
            .context("failed to list captures with context (paginated)")?;

        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            LEFT JOIN capture_run_items cri ON cri.capture_id = c.id
            WHERE ($1::text IS NULL OR s.slug = $1)
              AND ($2::text IS NULL OR c.scene_id = $2)
              AND ($3::text IS NULL OR c.status = $3)
              AND ($4::text IS NULL OR cri.run_id = $4)
            "#,
        )
        .bind(shader)
        .bind(scene)
        .bind(status)
        .bind(run_id)
        .fetch_one(db)
        .await
        .context("failed to count captures (paginated)")?;

        Ok((items, count.0))
    }

    /// Delete a capture
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM captures WHERE id = $1", id)
            .execute(executor)
            .await
            .context(format!("failed to delete capture '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Get the most recent completed non-outdated thumbnail per shader
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_thumbnails_by_shader(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<HashMap<String, String>> {
        struct Row {
            shader_id: String,
            image_url: String,
        }
        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT DISTINCT ON (sv.shader_id)
                sv.shader_id,
                c.image_url as "image_url!"
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            WHERE c.status = 'completed' AND c.outdated = FALSE AND c.image_url IS NOT NULL
            ORDER BY sv.shader_id, c.captured_at DESC NULLS LAST
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to batch fetch shader thumbnails")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.shader_id, r.image_url))
            .collect())
    }

    /// Get the most recent completed non-outdated thumbnail per scene
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_thumbnails_by_scene(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<HashMap<String, String>> {
        struct Row {
            scene_id: String,
            image_url: String,
        }
        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT DISTINCT ON (c.scene_id)
                c.scene_id,
                c.image_url as "image_url!"
            FROM captures c
            WHERE c.status = 'completed' AND c.outdated = FALSE AND c.image_url IS NOT NULL
            ORDER BY c.scene_id, c.captured_at DESC NULLS LAST
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to batch fetch scene thumbnails")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.scene_id, r.image_url))
            .collect())
    }

    /// Count completed captures per scene
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_count_by_scene(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<HashMap<String, i64>> {
        struct Row {
            scene_id: String,
            count: i64,
        }
        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT scene_id, COUNT(*) as "count!"
            FROM captures
            WHERE status = 'completed'
            GROUP BY scene_id
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to batch count captures by scene")?;

        Ok(rows.into_iter().map(|r| (r.scene_id, r.count)).collect())
    }
}
