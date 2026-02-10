use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppResult, OptionNotFoundExt};
use crate::id::{
    CaptureId, CaptureRunId, SceneId, SceneVersionId, ShaderId, ShaderVersionId, WorldVersionId,
};
use crate::models::{
    Capture, CaptureDetail, CaptureFreshness, CaptureRunStatus, CaptureStatus, CaptureWithContext,
};

pub struct ThumbnailInfo {
    pub image_url: String,
    pub thumbhash: Option<String>,
}

/// Optional filters for listing captures with context.
#[derive(Default)]
pub struct CaptureFilters {
    pub shader_id: Option<ShaderId>,
    pub shader_slug: Option<String>,
    pub scene_id: Option<SceneId>,
    pub version_id: Option<ShaderVersionId>,
    pub profile: Option<String>,
    pub status: Option<CaptureStatus>,
    pub run_id: Option<CaptureRunId>,
    pub scene_active: Option<bool>,
}

#[derive(Debug)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

/// Controls DISTINCT ON behavior for capture list queries.
#[derive(Debug)]
pub enum CaptureDistinct {
    /// No deduplication — return all matching rows.
    None,
    /// DISTINCT ON (shader_id) — one capture per shader (scene detail page).
    PerShader,
    /// DISTINCT ON (scene_id) — one capture per scene (shader detail page).
    PerScene,
}

pub struct CaptureRepo;

impl CaptureRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<Capture>> {
        sqlx::query_as!(
            Capture,
            r#"SELECT
                id AS "id: CaptureId",
                shader_version_id AS "shader_version_id: ShaderVersionId",
                scene_id AS "scene_id: SceneId",
                profile, image_url, image_path,
                video_url, avg_fps, min_fps, max_fps, frame_time_avg, frame_time_p99,
                minecraft_version, iris_version, gpu_model, resolution_width,
                resolution_height, captured_at,
                status AS "status!: CaptureStatus",
                error_message, thumbhash, file_size_bytes, content_type,
                world_version_id AS "world_version_id: WorldVersionId",
                scene_version_id AS "scene_version_id: SceneVersionId",
                created_at, updated_at
            FROM captures WHERE id = $1"#,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find capture '{}'", id))
        .map_err(Into::into)
    }

    /// List all completed captures (only from active scenes)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_completed(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<Capture>> {
        let captures = sqlx::query_as!(
            Capture,
            r#"
            SELECT
                c.id AS "id: CaptureId",
                c.shader_version_id AS "shader_version_id: ShaderVersionId",
                c.scene_id AS "scene_id: SceneId",
                c.profile, c.image_url,
                c.image_path, c.video_url, c.avg_fps, c.min_fps, c.max_fps,
                c.frame_time_avg, c.frame_time_p99, c.minecraft_version,
                c.iris_version, c.gpu_model, c.resolution_width, c.resolution_height,
                c.captured_at,
                c.status AS "status!: CaptureStatus",
                c.error_message, c.thumbhash, c.file_size_bytes, c.content_type,
                c.world_version_id AS "world_version_id: WorldVersionId",
                c.scene_version_id AS "scene_version_id: SceneVersionId",
                c.created_at, c.updated_at
            FROM captures c
            JOIN scenes sc ON c.scene_id = sc.id
            WHERE c.status = 'completed' AND sc.active = TRUE
            ORDER BY c.created_at DESC
            "#
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
            r#"SELECT
                id AS "id: CaptureId",
                shader_version_id AS "shader_version_id: ShaderVersionId",
                scene_id AS "scene_id: SceneId",
                profile, image_url, image_path,
                video_url, avg_fps, min_fps, max_fps, frame_time_avg, frame_time_p99,
                minecraft_version, iris_version, gpu_model, resolution_width,
                resolution_height, captured_at,
                status AS "status!: CaptureStatus",
                error_message, thumbhash, file_size_bytes, content_type,
                world_version_id AS "world_version_id: WorldVersionId",
                scene_version_id AS "scene_version_id: SceneVersionId",
                created_at, updated_at
            FROM captures WHERE shader_version_id = $1 ORDER BY created_at DESC"#,
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
            r#"SELECT
                id AS "id: CaptureId",
                shader_version_id AS "shader_version_id: ShaderVersionId",
                scene_id AS "scene_id: SceneId",
                profile, image_url, image_path,
                video_url, avg_fps, min_fps, max_fps, frame_time_avg, frame_time_p99,
                minecraft_version, iris_version, gpu_model, resolution_width,
                resolution_height, captured_at,
                status AS "status!: CaptureStatus",
                error_message, thumbhash, file_size_bytes, content_type,
                world_version_id AS "world_version_id: WorldVersionId",
                scene_version_id AS "scene_version_id: SceneVersionId",
                created_at, updated_at
            FROM captures WHERE scene_id = $1 ORDER BY created_at DESC"#,
            scene_id
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list captures for scene '{}'", scene_id))?;

        debug!(count = captures.len(), "Listed captures for scene");
        Ok(captures)
    }

    /// List captures with context, supporting flexible filtering, pagination, and deduplication.
    ///
    /// Replaces the former `list_all_with_context`, `list_all_with_context_paginated`,
    /// `list_with_context_for_scene`, `list_with_context_for_shader`, and
    /// `list_with_context_for_shader_filtered` methods.
    ///
    /// Returns `(captures, total_count)` where `total_count` is `Some` only when
    /// `pagination` is provided.
    #[instrument(skip(db, filters), level = "debug")]
    pub async fn list_with_context(
        db: &DbPool,
        filters: &CaptureFilters,
        pagination: Option<&Pagination>,
        distinct: CaptureDistinct,
    ) -> AppResult<(Vec<CaptureWithContext>, Option<i64>)> {
        let status_str = filters.status.map(|s| s.as_str().to_owned());
        let shader_id = filters.shader_id.as_ref().map(AsRef::<str>::as_ref);
        let scene_id = filters.scene_id.as_ref().map(AsRef::<str>::as_ref);
        let version_id = filters.version_id.as_ref().map(AsRef::<str>::as_ref);
        let run_id = filters.run_id.as_ref().map(AsRef::<str>::as_ref);

        let items = match distinct {
            CaptureDistinct::None => {
                if let Some(page) = pagination {
                    sqlx::query_as!(
                        CaptureWithContext,
                        r#"
                        SELECT
                            id AS "id!", scene_id AS "scene_id!",
                            shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                            shader_version AS "shader_version!",
                            profile, image_path, image_url, thumbhash, captured_at,
                            resolution_width, resolution_height, file_size_bytes,
                            run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                            shader_author, scene_name, scene_slug,
                            freshness AS "freshness!: CaptureFreshness"
                        FROM capture_contexts
                        WHERE ($1::text IS NULL OR shader_id = $1)
                          AND ($2::text IS NULL OR shader_slug = $2)
                          AND ($3::text IS NULL OR scene_id = $3)
                          AND ($4::text IS NULL OR shader_version_id = $4)
                          AND ($5::text IS NULL OR profile = $5)
                          AND ($6::text IS NULL OR capture_status = $6)
                          AND ($7::text IS NULL OR run_id = $7)
                          AND ($8::bool IS NULL OR scene_active = $8)
                        ORDER BY captured_at DESC NULLS LAST
                        LIMIT $9 OFFSET $10
                        "#,
                        shader_id,
                        filters.shader_slug,
                        scene_id,
                        version_id,
                        filters.profile,
                        status_str,
                        run_id,
                        filters.scene_active,
                        page.limit,
                        page.offset,
                    )
                    .fetch_all(db)
                    .await
                    .context("failed to list captures with context")?
                } else {
                    sqlx::query_as!(
                        CaptureWithContext,
                        r#"
                        SELECT
                            id AS "id!", scene_id AS "scene_id!",
                            shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                            shader_version AS "shader_version!",
                            profile, image_path, image_url, thumbhash, captured_at,
                            resolution_width, resolution_height, file_size_bytes,
                            run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                            shader_author, scene_name, scene_slug,
                            freshness AS "freshness!: CaptureFreshness"
                        FROM capture_contexts
                        WHERE ($1::text IS NULL OR shader_id = $1)
                          AND ($2::text IS NULL OR shader_slug = $2)
                          AND ($3::text IS NULL OR scene_id = $3)
                          AND ($4::text IS NULL OR shader_version_id = $4)
                          AND ($5::text IS NULL OR profile = $5)
                          AND ($6::text IS NULL OR capture_status = $6)
                          AND ($7::text IS NULL OR run_id = $7)
                          AND ($8::bool IS NULL OR scene_active = $8)
                        ORDER BY captured_at DESC NULLS LAST
                        "#,
                        shader_id,
                        filters.shader_slug,
                        scene_id,
                        version_id,
                        filters.profile,
                        status_str,
                        run_id,
                        filters.scene_active,
                    )
                    .fetch_all(db)
                    .await
                    .context("failed to list captures with context")?
                }
            }
            CaptureDistinct::PerShader => sqlx::query_as!(
                CaptureWithContext,
                r#"
                    SELECT DISTINCT ON (shader_id)
                        id AS "id!", scene_id AS "scene_id!",
                        shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                        shader_version AS "shader_version!",
                        profile, image_path, image_url, thumbhash, captured_at,
                        resolution_width, resolution_height, file_size_bytes,
                        run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                        shader_author, scene_name, scene_slug,
                        freshness AS "freshness!: CaptureFreshness"
                    FROM capture_contexts
                    WHERE ($1::text IS NULL OR shader_id = $1)
                      AND ($2::text IS NULL OR shader_slug = $2)
                      AND ($3::text IS NULL OR scene_id = $3)
                      AND ($4::text IS NULL OR shader_version_id = $4)
                      AND ($5::text IS NULL OR profile = $5)
                      AND ($6::text IS NULL OR capture_status = $6)
                      AND ($7::text IS NULL OR run_id = $7)
                      AND ($8::bool IS NULL OR scene_active = $8)
                    ORDER BY shader_id, captured_at DESC NULLS LAST
                    "#,
                shader_id,
                filters.shader_slug,
                scene_id,
                version_id,
                filters.profile,
                status_str,
                run_id,
                filters.scene_active,
            )
            .fetch_all(db)
            .await
            .context("failed to list captures with context (per-shader)")?,
            CaptureDistinct::PerScene => sqlx::query_as!(
                CaptureWithContext,
                r#"
                    SELECT DISTINCT ON (scene_id)
                        id AS "id!", scene_id AS "scene_id!",
                        shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                        shader_version AS "shader_version!",
                        profile, image_path, image_url, thumbhash, captured_at,
                        resolution_width, resolution_height, file_size_bytes,
                        run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                        shader_author, scene_name, scene_slug,
                        freshness AS "freshness!: CaptureFreshness"
                    FROM capture_contexts
                    WHERE ($1::text IS NULL OR shader_id = $1)
                      AND ($2::text IS NULL OR shader_slug = $2)
                      AND ($3::text IS NULL OR scene_id = $3)
                      AND ($4::text IS NULL OR shader_version_id = $4)
                      AND ($5::text IS NULL OR profile = $5)
                      AND ($6::text IS NULL OR capture_status = $6)
                      AND ($7::text IS NULL OR run_id = $7)
                      AND ($8::bool IS NULL OR scene_active = $8)
                    ORDER BY scene_id, captured_at DESC NULLS LAST
                    "#,
                shader_id,
                filters.shader_slug,
                scene_id,
                version_id,
                filters.profile,
                status_str,
                run_id,
                filters.scene_active,
            )
            .fetch_all(db)
            .await
            .context("failed to list captures with context (per-scene)")?,
        };

        let count = if let Some(_page) = pagination {
            let total = sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) AS "count!"
                FROM capture_contexts
                WHERE ($1::text IS NULL OR shader_id = $1)
                  AND ($2::text IS NULL OR shader_slug = $2)
                  AND ($3::text IS NULL OR scene_id = $3)
                  AND ($4::text IS NULL OR shader_version_id = $4)
                  AND ($5::text IS NULL OR profile = $5)
                  AND ($6::text IS NULL OR capture_status = $6)
                  AND ($7::text IS NULL OR run_id = $7)
                  AND ($8::bool IS NULL OR scene_active = $8)
                "#,
                shader_id,
                filters.shader_slug,
                scene_id,
                version_id,
                filters.profile,
                status_str,
                run_id,
                filters.scene_active,
            )
            .fetch_one(db)
            .await
            .context("failed to count captures with context")?;
            Some(total)
        } else {
            None
        };

        debug!(count = items.len(), "Listed captures with context");
        Ok((items, count))
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
        world_version_id: Option<&str>,
        scene_version_id: Option<&str>,
        status: CaptureStatus,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO captures (
                id, shader_version_id, scene_id, profile, image_path, image_url,
                resolution_width, resolution_height, world_version_id, scene_version_id,
                status, created_at, updated_at, captured_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12, $12)
            "#,
            id,
            shader_version_id,
            scene_id,
            profile,
            image_path,
            image_url,
            resolution_width,
            resolution_height,
            world_version_id,
            scene_version_id,
            status as CaptureStatus,
            captured_at
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to insert capture for scene '{}' with shader version '{}'",
            scene_id, shader_version_id
        ))?;

        debug!(scene_id, shader_version_id, %status, "Capture inserted");
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
    /// Get a single capture with context (for admin detail view)
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_with_context(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<CaptureWithContext> {
        sqlx::query_as!(
            CaptureWithContext,
            r#"
            SELECT
                id AS "id!", scene_id AS "scene_id!",
                shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                shader_version AS "shader_version!",
                profile, image_path, image_url, thumbhash, captured_at,
                resolution_width, resolution_height, file_size_bytes,
                run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                shader_author, scene_name, scene_slug,
                freshness AS "freshness!: CaptureFreshness"
            FROM capture_contexts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to get capture with context '{}'", id))?
        .or_not_found("Capture", id)
    }

    /// Full capture detail with technical metadata and related captures.
    /// Combines the full `captures` row with context joins and fetches
    /// three groups of related captures for cross-referencing.
    #[instrument(skip(db), level = "debug")]
    pub async fn get_detail(db: &DbPool, id: &str) -> AppResult<CaptureDetail> {
        #[derive(FromRow)]
        struct DetailRow {
            // Context fields
            id: CaptureId,
            scene_id: SceneId,
            shader_slug: String,
            shader_name: String,
            shader_version: String,
            shader_version_id: ShaderVersionId,
            profile: Option<String>,
            image_path: Option<String>,
            image_url: Option<String>,
            thumbhash: Option<String>,
            captured_at: Option<DateTime<Utc>>,
            resolution_width: Option<i32>,
            resolution_height: Option<i32>,
            file_size_bytes: Option<i64>,
            run_id: Option<CaptureRunId>,
            run_status: Option<CaptureRunStatus>,
            shader_author: Option<String>,
            scene_name: Option<String>,
            scene_slug: Option<String>,
            freshness: CaptureFreshness,
            // Technical metadata
            status: CaptureStatus,
            error_message: Option<String>,
            video_url: Option<String>,
            avg_fps: Option<f64>,
            min_fps: Option<f64>,
            max_fps: Option<f64>,
            frame_time_avg: Option<f64>,
            frame_time_p99: Option<f64>,
            minecraft_version: Option<String>,
            iris_version: Option<String>,
            gpu_model: Option<String>,
            content_type: Option<String>,
            world_version_id: Option<WorldVersionId>,
            scene_version_id: Option<SceneVersionId>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as!(
            DetailRow,
            r#"
            SELECT
                id AS "id!: CaptureId", scene_id AS "scene_id!: SceneId",
                shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                shader_version AS "shader_version!",
                shader_version_id AS "shader_version_id!: ShaderVersionId",
                profile, image_path, image_url, thumbhash,
                captured_at, resolution_width, resolution_height, file_size_bytes,
                run_id AS "run_id: CaptureRunId",
                run_status AS "run_status: CaptureRunStatus",
                shader_author, scene_name, scene_slug,
                freshness AS "freshness!: CaptureFreshness",
                status AS "status!: CaptureStatus", error_message, video_url,
                avg_fps, min_fps, max_fps, frame_time_avg, frame_time_p99,
                minecraft_version, iris_version, gpu_model, content_type,
                world_version_id AS "world_version_id: WorldVersionId",
                scene_version_id AS "scene_version_id: SceneVersionId",
                created_at AS "created_at!", updated_at AS "updated_at!"
            FROM capture_details
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(db)
        .await
        .context(format!("failed to get capture detail '{}'", id))?
        .or_not_found("Capture", id)?;

        // Fetch related captures in parallel
        let scene_id_str = row.scene_id.0.clone();
        let shader_version_id_str = row.shader_version_id.0.clone();
        let capture_id_str = row.id.0.clone();
        let run_id_str = row.run_id.as_ref().map(|r| r.0.clone());

        // Same shader + same scene (different profiles/versions of same shader in same scene)
        let same_shader_scene_fut = sqlx::query_as!(
            CaptureWithContext,
            r#"
            SELECT
                id AS "id!", scene_id AS "scene_id!",
                shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                shader_version AS "shader_version!",
                profile, image_path, image_url, thumbhash, captured_at,
                resolution_width, resolution_height, file_size_bytes,
                run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                shader_author, scene_name, scene_slug,
                freshness AS "freshness!: CaptureFreshness"
            FROM capture_contexts
            WHERE scene_id = $1
              AND shader_id = (SELECT shader_id FROM shader_versions WHERE id = $2)
              AND id != $3
              AND capture_status = 'completed'
            ORDER BY captured_at DESC NULLS LAST
            LIMIT 8
            "#,
            scene_id_str,
            shader_version_id_str,
            capture_id_str,
        )
        .fetch_all(db);

        // Same scene, different shaders
        let same_scene_fut = sqlx::query_as!(
            CaptureWithContext,
            r#"
            SELECT
                id AS "id!", scene_id AS "scene_id!",
                shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                shader_version AS "shader_version!",
                profile, image_path, image_url, thumbhash, captured_at,
                resolution_width, resolution_height, file_size_bytes,
                run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                shader_author, scene_name, scene_slug,
                freshness AS "freshness!: CaptureFreshness"
            FROM capture_contexts
            WHERE scene_id = $1
              AND shader_id != (SELECT shader_id FROM shader_versions WHERE id = $2)
              AND id != $3
              AND capture_status = 'completed'
            ORDER BY captured_at DESC NULLS LAST
            LIMIT 8
            "#,
            scene_id_str,
            shader_version_id_str,
            capture_id_str,
        )
        .fetch_all(db);

        let (same_shader_scene, same_scene, same_run) = if let Some(ref rid) = run_id_str {
            let same_run_fut = sqlx::query_as!(
                CaptureWithContext,
                r#"
                SELECT
                    id AS "id!", scene_id AS "scene_id!",
                    shader_slug AS "shader_slug!", shader_name AS "shader_name!",
                    shader_version AS "shader_version!",
                    profile, image_path, image_url, thumbhash, captured_at,
                    resolution_width, resolution_height, file_size_bytes,
                run_id AS "run_id: CaptureRunId", run_status AS "run_status: CaptureRunStatus",
                shader_author, scene_name, scene_slug,
                freshness AS "freshness!: CaptureFreshness"
            FROM capture_contexts
            WHERE run_id = $1
              AND id != $2
                  AND capture_status = 'completed'
                ORDER BY captured_at DESC NULLS LAST
                LIMIT 8
                "#,
                rid,
                capture_id_str,
            )
            .fetch_all(db);

            let (ss, sc, sr) =
                tokio::try_join!(same_shader_scene_fut, same_scene_fut, same_run_fut)
                    .context("failed to fetch related captures")?;
            (ss, sc, sr)
        } else {
            let (ss, sc) = tokio::try_join!(same_shader_scene_fut, same_scene_fut)
                .context("failed to fetch related captures")?;
            (ss, sc, Vec::new())
        };

        let context = CaptureWithContext {
            id: row.id,
            scene_id: row.scene_id,
            shader_slug: row.shader_slug,
            shader_name: row.shader_name,
            shader_version: row.shader_version,
            profile: row.profile,
            image_path: row.image_path,
            image_url: row.image_url,
            thumbhash: row.thumbhash,
            captured_at: row.captured_at,
            resolution_width: row.resolution_width,
            resolution_height: row.resolution_height,
            file_size_bytes: row.file_size_bytes,
            run_id: row.run_id,
            run_status: row.run_status,
            shader_author: row.shader_author,
            scene_name: row.scene_name,
            scene_slug: row.scene_slug,
            freshness: row.freshness,
        };

        Ok(CaptureDetail {
            context,
            shader_version_id: row.shader_version_id,
            status: row.status,
            error_message: row.error_message,
            video_url: row.video_url,
            avg_fps: row.avg_fps,
            min_fps: row.min_fps,
            max_fps: row.max_fps,
            frame_time_avg: row.frame_time_avg,
            frame_time_p99: row.frame_time_p99,
            minecraft_version: row.minecraft_version,
            iris_version: row.iris_version,
            gpu_model: row.gpu_model,
            content_type: row.content_type,
            world_version_id: row.world_version_id,
            scene_version_id: row.scene_version_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            same_shader_scene,
            same_scene,
            same_run,
        })
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

    /// Get the most recent completed non-outdated thumbnail per shader (active scenes only)
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
            JOIN scenes sc ON c.scene_id = sc.id
            WHERE c.status = 'completed' AND c.image_url IS NOT NULL AND sc.active = TRUE
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

    /// Get the most recent completed non-outdated thumbnail per scene (active scenes only)
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
            JOIN scenes sc ON c.scene_id = sc.id
            WHERE c.status = 'completed' AND c.image_url IS NOT NULL AND sc.active = TRUE
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

    /// Update image fields after transcoding a capture from PNG to WebP
    #[instrument(skip(executor), level = "debug")]
    pub async fn update_image_after_transcode(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        image_url: &str,
        image_path: Option<&str>,
        file_size_bytes: i64,
        content_type: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE captures
            SET image_url = $2, image_path = $3, file_size_bytes = $4, content_type = $5, updated_at = now()
            WHERE id = $1
            "#,
            id,
            image_url,
            image_path,
            file_size_bytes,
            content_type
        )
        .execute(executor)
        .await
        .context(format!("failed to update image after transcode for '{}'", id))?;

        debug!(id, "Updated capture image after transcode");
        Ok(())
    }

    /// List IDs of completed captures still stored as PNG (for WebP transcoding backfill)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_png_capture_ids(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<String>> {
        let ids = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM captures
            WHERE status = 'completed'
              AND image_url IS NOT NULL
              AND (content_type = 'image/png' OR image_url LIKE '%.png')
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to list PNG capture ids for transcoding")?;

        Ok(ids)
    }

    /// Count completed captures per scene (active scenes only)
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
            SELECT c.scene_id, COUNT(*) as "count!"
            FROM captures c
            JOIN scenes sc ON c.scene_id = sc.id
            WHERE c.status = 'completed' AND sc.active = TRUE
            GROUP BY c.scene_id
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
