use anyhow::Context;
use chrono::{DateTime, Utc};
use tracing::{debug, instrument, trace};

use crate::error::AppResult;
use crate::models::PendingUpload;

pub struct PendingUploadRepo;

impl PendingUploadRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        upload_id: &str,
    ) -> AppResult<Option<PendingUpload>> {
        sqlx::query_as!(
            PendingUpload,
            "SELECT * FROM pending_uploads WHERE upload_id = $1",
            upload_id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find pending upload '{}'", upload_id))
        .map_err(Into::into)
    }

    /// Create a pending upload for world creation (slug/name/minecraft_version required).
    #[instrument(skip(executor), level = "debug")]
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        upload_id: &str,
        slug: &str,
        name: &str,
        description: Option<&str>,
        minecraft_version: &str,
        file_hash: &str,
        size_bytes: i64,
        upload_key: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO pending_uploads (upload_id, slug, name, description, minecraft_version, file_hash, size_bytes, upload_key, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
            "#,
            upload_id,
            slug,
            name,
            description,
            minecraft_version,
            file_hash,
            size_bytes,
            upload_key,
            expires_at
        )
        .execute(executor)
        .await
        .context(format!("failed to create pending upload '{}'", upload_id))?;

        debug!("Pending upload created");
        Ok(())
    }

    /// Create a pending upload for a new version of an existing world.
    #[instrument(skip(executor), level = "debug")]
    pub async fn create_for_version(
        executor: impl sqlx::PgExecutor<'_>,
        upload_id: &str,
        world_id: &str,
        file_hash: &str,
        size_bytes: i64,
        upload_key: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO pending_uploads (upload_id, world_id, file_hash, size_bytes, upload_key, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, now())
            "#,
            upload_id,
            world_id,
            file_hash,
            size_bytes,
            upload_key,
            expires_at
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to create pending version upload '{}'",
            upload_id
        ))?;

        debug!("Pending version upload created");
        Ok(())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, upload_id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM pending_uploads WHERE upload_id = $1",
            upload_id
        )
        .execute(executor)
        .await
        .context(format!("failed to delete pending upload '{}'", upload_id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Find all expired pending uploads
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_expired(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<ExpiredUpload>> {
        let expired = sqlx::query_as!(
            ExpiredUpload,
            r#"
            SELECT upload_id, upload_key
            FROM pending_uploads
            WHERE expires_at < now()
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to list expired uploads")?;

        if !expired.is_empty() {
            debug!(count = expired.len(), "Found expired uploads");
        }
        Ok(expired)
    }

    /// Delete a single expired upload by ID
    #[instrument(skip(executor), level = "trace")]
    pub async fn delete_expired_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        upload_id: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM pending_uploads WHERE upload_id = $1",
            upload_id
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to delete expired pending upload '{}'",
            upload_id
        ))?;

        if result.rows_affected() > 0 {
            trace!("Deleted expired pending upload");
        }
        Ok(result.rows_affected() > 0)
    }
}

/// Minimal struct for cleanup operations
pub struct ExpiredUpload {
    pub upload_id: String,
    pub upload_key: String,
}
