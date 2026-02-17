use anyhow::Context;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::id::PendingSceneUploadId;
use crate::models::PendingSceneUpload;

pub struct PendingSceneUploadRepo;

impl PendingSceneUploadRepo {
    /// Create a pending upload for a NEW scene (scene doesn't exist yet).
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(executor), level = "debug")]
    pub async fn create_for_new_scene(
        executor: impl sqlx::PgExecutor<'_>,
        id: &PendingSceneUploadId,
        scene_name: &str,
        scene_slug: &str,
        scene_dimension: &str,
        scene_description: Option<&str>,
        minecraft_version: &str,
        file_hash: &str,
        size_bytes: i64,
        r2_key: &str,
    ) -> AppResult<PendingSceneUpload> {
        sqlx::query_as!(
            PendingSceneUpload,
            r#"
            INSERT INTO pending_scene_uploads
                (id, scene_name, scene_slug, scene_dimension, scene_description,
                 minecraft_version, file_hash, size_bytes, r2_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            id.as_ref(),
            scene_name,
            scene_slug,
            scene_dimension,
            scene_description,
            minecraft_version,
            file_hash,
            size_bytes,
            r2_key,
        )
        .fetch_one(executor)
        .await
        .context("failed to create pending upload for new scene")
        .map_err(Into::into)
    }

    /// Create a pending upload for an EXISTING scene (new version).
    #[instrument(skip(executor), level = "debug")]
    pub async fn create_for_existing_scene(
        executor: impl sqlx::PgExecutor<'_>,
        id: &PendingSceneUploadId,
        scene_id: &str,
        minecraft_version: &str,
        file_hash: &str,
        size_bytes: i64,
        r2_key: &str,
    ) -> AppResult<PendingSceneUpload> {
        sqlx::query_as!(
            PendingSceneUpload,
            r#"
            INSERT INTO pending_scene_uploads
                (id, scene_id, minecraft_version, file_hash, size_bytes, r2_key)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            id.as_ref(),
            scene_id,
            minecraft_version,
            file_hash,
            size_bytes,
            r2_key,
        )
        .fetch_one(executor)
        .await
        .context("failed to create pending upload for existing scene")
        .map_err(Into::into)
    }

    /// Find a non-expired pending upload by ID.
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<PendingSceneUpload>> {
        sqlx::query_as!(
            PendingSceneUpload,
            r#"
            SELECT * FROM pending_scene_uploads
            WHERE id = $1 AND expires_at > now()
            "#,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find pending upload '{}'", id))
        .map_err(Into::into)
    }

    /// Delete a pending upload by ID (after successful confirmation).
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM pending_scene_uploads WHERE id = $1", id)
            .execute(executor)
            .await
            .context(format!("failed to delete pending upload '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete expired pending uploads. Returns count of rows deleted.
    #[instrument(skip(pool), level = "debug")]
    pub async fn cleanup_expired(pool: &DbPool) -> AppResult<u64> {
        let result = sqlx::query!("DELETE FROM pending_scene_uploads WHERE expires_at <= now()")
            .execute(pool)
            .await
            .context("failed to cleanup expired pending uploads")?;

        let count = result.rows_affected();
        if count > 0 {
            debug!(count, "Cleaned up expired pending scene uploads");
        }
        Ok(count)
    }
}
