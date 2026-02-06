use anyhow::Context;
use chrono::{DateTime, Utc};
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{Capture, CaptureWithContext};

pub struct CaptureRepo;

impl CaptureRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: &str) -> AppResult<Option<Capture>> {
        sqlx::query_as!(Capture, "SELECT * FROM captures WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find capture '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: &str) -> AppResult<Capture> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Capture '{}' not found", id)))
    }

    /// List all completed captures
    #[instrument(skip(db), level = "debug")]
    pub async fn list_completed(db: &DbPool) -> AppResult<Vec<Capture>> {
        let captures = sqlx::query_as!(
            Capture,
            "SELECT * FROM captures WHERE status = 'completed' ORDER BY created_at DESC"
        )
        .fetch_all(db)
        .await
        .context("failed to list completed captures")?;

        debug!(count = captures.len(), "Listed completed captures");
        Ok(captures)
    }

    /// List captures by shader version
    #[instrument(skip(db), level = "debug")]
    pub async fn list_by_shader_version(
        db: &DbPool,
        shader_version_id: &str,
    ) -> AppResult<Vec<Capture>> {
        let captures = sqlx::query_as!(
            Capture,
            "SELECT * FROM captures WHERE shader_version_id = $1 ORDER BY created_at DESC",
            shader_version_id
        )
        .fetch_all(db)
        .await
        .context(format!(
            "failed to list captures for shader version '{}'",
            shader_version_id
        ))?;

        debug!(count = captures.len(), "Listed captures for shader version");
        Ok(captures)
    }

    /// List captures by scene
    #[instrument(skip(db), level = "debug")]
    pub async fn list_by_scene(db: &DbPool, scene_id: &str) -> AppResult<Vec<Capture>> {
        let captures = sqlx::query_as!(
            Capture,
            "SELECT * FROM captures WHERE scene_id = $1 ORDER BY created_at DESC",
            scene_id
        )
        .fetch_all(db)
        .await
        .context(format!("failed to list captures for scene '{}'", scene_id))?;

        debug!(count = captures.len(), "Listed captures for scene");
        Ok(captures)
    }

    /// Fetch captures with shader/version context for a scene (for scene detail page)
    #[instrument(skip(db), level = "debug")]
    pub async fn list_with_context_for_scene(
        db: &DbPool,
        scene_id: &str,
    ) -> AppResult<Vec<CaptureWithContext>> {
        let captures = sqlx::query_as!(
            CaptureWithContext,
            r#"
            SELECT
                c.id,
                c.scene_id,
                s.slug as shader_slug,
                s.name as shader_name,
                sv.version as shader_version,
                c.profile,
                c.screenshot_path,
                c.screenshot_url,
                c.captured_at,
                c.resolution_width,
                c.resolution_height
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            WHERE c.scene_id = $1 AND c.status = 'completed'
            ORDER BY s.name, sv.created_at DESC
            "#,
            scene_id
        )
        .fetch_all(db)
        .await
        .context(format!(
            "failed to get captures with context for scene '{}'",
            scene_id
        ))?;

        debug!(count = captures.len(), "Fetched captures with context");
        Ok(captures)
    }

    /// Create or update a capture (upsert on shader_version_id + scene_id + profile)
    #[instrument(skip(db), level = "debug")]
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        db: &DbPool,
        id: &str,
        shader_version_id: &str,
        scene_id: &str,
        profile: Option<&str>,
        screenshot_path: Option<&str>,
        screenshot_url: Option<&str>,
        resolution_width: Option<i32>,
        resolution_height: Option<i32>,
        captured_at: Option<DateTime<Utc>>,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO captures (
                id, shader_version_id, scene_id, profile, screenshot_path, screenshot_url,
                resolution_width, resolution_height, outdated, status, created_at, updated_at, captured_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, FALSE, 'completed', $9, $9, $9)
            ON CONFLICT (shader_version_id, scene_id, profile)
            DO UPDATE SET
                screenshot_path = excluded.screenshot_path,
                screenshot_url = excluded.screenshot_url,
                resolution_width = excluded.resolution_width,
                resolution_height = excluded.resolution_height,
                outdated = FALSE,
                status = 'completed',
                updated_at = excluded.updated_at,
                captured_at = excluded.captured_at
            "#,
            id,
            shader_version_id,
            scene_id,
            profile,
            screenshot_path,
            screenshot_url,
            resolution_width,
            resolution_height,
            captured_at
        )
        .execute(db)
        .await
        .context(format!(
            "failed to upsert capture for scene '{}' with shader version '{}'",
            scene_id, shader_version_id
        ))?;

        debug!(scene_id, shader_version_id, "Capture upserted");
        Ok(())
    }

    /// Mark captures as outdated for a scene (when scene is updated)
    #[instrument(skip(db), level = "debug")]
    pub async fn mark_outdated_for_scene(db: &DbPool, scene_id: &str) -> AppResult<u64> {
        let result = sqlx::query!(
            "UPDATE captures SET outdated = TRUE WHERE scene_id = $1 AND status = 'completed'",
            scene_id
        )
        .execute(db)
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
    #[instrument(skip(db), level = "debug")]
    pub async fn count_by_status(db: &DbPool) -> AppResult<CaptureStatusCounts> {
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
        .fetch_one(db)
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
    #[instrument(skip(db), level = "debug")]
    pub async fn list_all_with_context(db: &DbPool) -> AppResult<Vec<CaptureWithContext>> {
        let captures = sqlx::query_as!(
            CaptureWithContext,
            r#"
            SELECT
                c.id,
                c.scene_id,
                s.slug as shader_slug,
                s.name as shader_name,
                sv.version as shader_version,
                c.profile,
                c.screenshot_path,
                c.screenshot_url,
                c.captured_at,
                c.resolution_width,
                c.resolution_height
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            ORDER BY c.created_at DESC
            "#
        )
        .fetch_all(db)
        .await
        .context("failed to list all captures with context")?;

        debug!(count = captures.len(), "Listed all captures with context");
        Ok(captures)
    }

    /// Get a single capture with context (for admin detail view)
    #[instrument(skip(db), level = "debug")]
    pub async fn get_with_context(db: &DbPool, id: &str) -> AppResult<CaptureWithContext> {
        sqlx::query_as!(
            CaptureWithContext,
            r#"
            SELECT
                c.id,
                c.scene_id,
                s.slug as shader_slug,
                s.name as shader_name,
                sv.version as shader_version,
                c.profile,
                c.screenshot_path,
                c.screenshot_url,
                c.captured_at,
                c.resolution_width,
                c.resolution_height
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            WHERE c.id = $1
            "#,
            id
        )
        .fetch_optional(db)
        .await
        .context(format!("failed to get capture with context '{}'", id))?
        .ok_or_else(|| AppError::NotFound(format!("Capture '{}' not found", id)))
    }

    /// Delete a capture
    #[instrument(skip(db), level = "debug")]
    pub async fn delete(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM captures WHERE id = $1", id)
            .execute(db)
            .await
            .context(format!("failed to delete capture '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }
}
