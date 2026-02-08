use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{Capture, CaptureWithContext};

pub struct ThumbnailInfo {
    pub image_url: String,
    pub thumbhash: Option<String>,
}

pub struct CaptureRepo;

/// Compile-time checked query returning `CaptureWithContext`.
///
/// SQLx macros require string-literal concatenation (`"a" + "b"`), so this
/// macro injects the shared SELECT/FROM/JOIN fragment and appends a caller-
/// supplied suffix (WHERE, ORDER BY, LIMIT, etc.).
///
/// # Variants
///
/// - `capture_ctx_query!($suffix, $args...)` — plain SELECT
/// - `capture_ctx_query!(distinct: $expr, $suffix, $args...)` — SELECT DISTINCT ON ($expr)
#[macro_export]
macro_rules! capture_ctx_query {
    ($suffix:literal $(, $arg:expr)* $(,)?) => {
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
                c.image_path,
                c.image_url,
                c.thumbhash,
                c.captured_at,
                c.resolution_width,
                c.resolution_height,
                c.file_size_bytes,
                cri.run_id as "run_id?: String",
                cr.status as "run_status?: String",
                (SELECT sa.name FROM shader_authors sa WHERE sa.shader_id = s.id LIMIT 1) as shader_author,
                sc.name as "scene_name?: String",
                sc.slug as "scene_slug?: String"
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            LEFT JOIN capture_run_items cri ON cri.capture_id = c.id
            LEFT JOIN capture_runs cr ON cri.run_id = cr.id
            LEFT JOIN scenes sc ON c.scene_id = sc.id
            "# + $suffix
            $(, $arg)*
        )
    };
    (distinct: $distinct:literal, $suffix:literal $(, $arg:expr)* $(,)?) => {
        sqlx::query_as!(
            CaptureWithContext,
            r#"SELECT DISTINCT ON ("# + $distinct + r#")
                c.id,
                c.scene_id,
                s.slug as shader_slug,
                s.name as shader_name,
                sv.version as shader_version,
                c.profile,
                c.image_path,
                c.image_url,
                c.thumbhash,
                c.captured_at,
                c.resolution_width,
                c.resolution_height,
                c.file_size_bytes,
                cri.run_id as "run_id?: String",
                cr.status as "run_status?: String",
                (SELECT sa.name FROM shader_authors sa WHERE sa.shader_id = s.id LIMIT 1) as shader_author,
                sc.name as "scene_name?: String",
                sc.slug as "scene_slug?: String"
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            LEFT JOIN capture_run_items cri ON cri.capture_id = c.id
            LEFT JOIN capture_runs cr ON cri.run_id = cr.id
            LEFT JOIN scenes sc ON c.scene_id = sc.id
            "# + $suffix
            $(, $arg)*
        )
    };
}

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

    /// Fetch the latest capture per shader for a scene (for scene detail page)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_with_context_for_scene(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<Vec<CaptureWithContext>> {
        let captures = capture_ctx_query!(
            distinct: "sv.shader_id",
            r#"
            WHERE c.scene_id = $1 AND c.status = 'completed'
            ORDER BY sv.shader_id, c.captured_at DESC NULLS LAST
            "#,
            scene_id
        )
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

#[derive(Debug, Serialize)]
pub struct StorageStats {
    pub total_bytes: i64,
    pub capture_count: i64,
    pub avg_bytes: i64,
    pub missing_count: i64,
}

#[derive(Debug, Serialize)]
pub struct StorageBucket {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub date: DateTime<Utc>,
    pub cumulative_bytes: i64,
    pub cumulative_count: i64,
    pub bucket_bytes: i64,
}

impl CaptureRepo {
    /// List all captures with context (for admin dashboard)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_all_with_context(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<CaptureWithContext>> {
        let captures = capture_ctx_query!(" ORDER BY c.created_at DESC")
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
        capture_ctx_query!(" WHERE c.id = $1", id)
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
        let items = capture_ctx_query!(
            r#"
            WHERE ($1::text IS NULL OR s.slug = $1)
              AND ($2::text IS NULL OR c.scene_id = $2)
              AND ($3::text IS NULL OR c.status = $3)
              AND ($4::text IS NULL OR cri.run_id = $4)
            ORDER BY c.created_at DESC
            LIMIT $5 OFFSET $6
            "#,
            shader,
            scene,
            status,
            run_id,
            limit,
            offset,
        )
        .fetch_all(db)
        .await
        .context("failed to list captures with context (paginated)")?;

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            LEFT JOIN capture_run_items cri ON cri.capture_id = c.id
            WHERE ($1::text IS NULL OR s.slug = $1)
              AND ($2::text IS NULL OR c.scene_id = $2)
              AND ($3::text IS NULL OR c.status = $3)
              AND ($4::text IS NULL OR cri.run_id = $4)
            "#,
            shader,
            scene,
            status,
            run_id,
        )
        .fetch_one(db)
        .await
        .context("failed to count captures (paginated)")?;

        Ok((items, count))
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
    ) -> AppResult<HashMap<String, ThumbnailInfo>> {
        struct Row {
            shader_id: String,
            image_url: String,
            thumbhash: Option<String>,
        }
        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT DISTINCT ON (sv.shader_id)
                sv.shader_id,
                c.image_url as "image_url!",
                c.thumbhash
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
            .map(|r| {
                (
                    r.shader_id,
                    ThumbnailInfo {
                        image_url: r.image_url,
                        thumbhash: r.thumbhash,
                    },
                )
            })
            .collect())
    }

    /// Get the most recent completed non-outdated thumbnail per scene
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_thumbnails_by_scene(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<HashMap<String, ThumbnailInfo>> {
        struct Row {
            scene_id: String,
            image_url: String,
            thumbhash: Option<String>,
        }
        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT DISTINCT ON (c.scene_id)
                c.scene_id,
                c.image_url as "image_url!",
                c.thumbhash
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
            .map(|r| {
                (
                    r.scene_id,
                    ThumbnailInfo {
                        image_url: r.image_url,
                        thumbhash: r.thumbhash,
                    },
                )
            })
            .collect())
    }

    /// List IDs of completed captures that are missing thumbhash OR file size metadata
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_unprocessed_ids(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<String>> {
        let ids = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM captures
            WHERE status = 'completed'
              AND image_url IS NOT NULL
              AND (thumbhash IS NULL OR file_size_bytes IS NULL)
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to list unprocessed capture ids")?;

        Ok(ids)
    }

    /// Set the thumbhash for a capture
    #[instrument(skip(executor), level = "debug")]
    pub async fn set_thumbhash(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        thumbhash: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "UPDATE captures SET thumbhash = $2 WHERE id = $1",
            id,
            thumbhash
        )
        .execute(executor)
        .await
        .context(format!("failed to set thumbhash for capture '{}'", id))?;

        Ok(())
    }

    /// Set file metadata (size and content type) for a capture
    #[instrument(skip(executor), level = "debug")]
    pub async fn set_file_metadata(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        file_size_bytes: i64,
        content_type: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "UPDATE captures SET file_size_bytes = $2, content_type = $3 WHERE id = $1",
            id,
            file_size_bytes,
            content_type
        )
        .execute(executor)
        .await
        .context(format!("failed to set file metadata for capture '{}'", id))?;

        Ok(())
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

    /// Get aggregate storage statistics for completed captures
    #[instrument(skip(executor), level = "debug")]
    pub async fn storage_stats(executor: impl sqlx::PgExecutor<'_>) -> AppResult<StorageStats> {
        let row = sqlx::query!(
            r#"
            SELECT
                COALESCE(SUM(file_size_bytes), 0)::int8 as "total_bytes!",
                COUNT(*) FILTER (WHERE file_size_bytes IS NOT NULL)::int8 as "capture_count!",
                COALESCE(AVG(file_size_bytes), 0)::int8 as "avg_bytes!",
                COUNT(*) FILTER (WHERE status = 'completed' AND image_url IS NOT NULL AND file_size_bytes IS NULL)::int8 as "missing_count!"
            FROM captures
            WHERE status = 'completed'
            "#
        )
        .fetch_one(executor)
        .await
        .context("failed to get storage stats")?;

        Ok(StorageStats {
            total_bytes: row.total_bytes,
            capture_count: row.capture_count,
            avg_bytes: row.avg_bytes,
            missing_count: row.missing_count,
        })
    }

    /// Get cumulative storage growth with gap-filled time series.
    ///
    /// Generates a continuous series at the given interval (in hours), starting
    /// from the earliest completed capture within the date range. Buckets with
    /// no captures are filled with zeros; cumulative values carry forward.
    #[instrument(skip(executor), level = "debug")]
    pub async fn storage_growth(
        executor: impl sqlx::PgExecutor<'_>,
        days: i32,
        interval_hours: i32,
    ) -> AppResult<Vec<StorageBucket>> {
        struct Row {
            date: DateTime<Utc>,
            cumulative_bytes: i64,
            cumulative_count: i64,
            bucket_bytes: i64,
        }
        let rows = sqlx::query_as!(
            Row,
            r#"
            WITH
              bounds AS (
                SELECT
                  date_trunc('hour', MIN(created_at)) AS first_ts
                FROM captures
                WHERE status = 'completed'
                  AND created_at >= now() - make_interval(days => $1)
              ),
              buckets AS (
                SELECT generate_series(
                  (SELECT first_ts FROM bounds),
                  now(),
                  make_interval(hours => $2)
                ) AS bucket
              ),
              capture_agg AS (
                SELECT
                  -- Align each capture to its bucket start
                  (SELECT MAX(b.bucket) FROM buckets b WHERE b.bucket <= c.created_at) AS bucket,
                  COALESCE(SUM(file_size_bytes), 0)::int8 AS bytes,
                  COUNT(*)::int8 AS cnt
                FROM captures c
                WHERE c.status = 'completed'
                  AND c.created_at >= (SELECT first_ts FROM bounds)
                GROUP BY 1
              )
            SELECT
              b.bucket AS "date!",
              (SUM(COALESCE(ca.bytes, 0)) OVER w)::int8 AS "cumulative_bytes!",
              (SUM(COALESCE(ca.cnt, 0)) OVER w)::int8 AS "cumulative_count!",
              COALESCE(ca.bytes, 0)::int8 AS "bucket_bytes!"
            FROM buckets b
            LEFT JOIN capture_agg ca ON ca.bucket = b.bucket
            WHERE (SELECT first_ts FROM bounds) IS NOT NULL
            WINDOW w AS (ORDER BY b.bucket)
            ORDER BY b.bucket
            "#,
            days,
            interval_hours
        )
        .fetch_all(executor)
        .await
        .context("failed to get storage growth")?;

        Ok(rows
            .into_iter()
            .map(|r| StorageBucket {
                date: r.date,
                cumulative_bytes: r.cumulative_bytes,
                cumulative_count: r.cumulative_count,
                bucket_bytes: r.bucket_bytes,
            })
            .collect())
    }
}
