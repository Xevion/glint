use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::AppResult;
use crate::models::WorldVersion;

pub struct WorldVersionRepo;

impl WorldVersionRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        world_id: &str,
        file_url: &str,
        file_hash: &str,
        size_bytes: i64,
    ) -> AppResult<WorldVersion> {
        sqlx::query_as!(
            WorldVersion,
            r#"
            INSERT INTO world_versions (id, world_id, file_url, file_hash, size_bytes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            id,
            world_id,
            file_url,
            file_hash,
            size_bytes,
        )
        .fetch_one(executor)
        .await
        .context(format!(
            "failed to create world version for world '{}'",
            world_id
        ))
        .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_latest_for_world(
        executor: impl sqlx::PgExecutor<'_>,
        world_id: &str,
    ) -> AppResult<Option<WorldVersion>> {
        sqlx::query_as!(
            WorldVersion,
            r#"
            SELECT * FROM world_versions
            WHERE world_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            world_id,
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to get latest world version for '{}'",
            world_id
        ))
        .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_world(
        executor: impl sqlx::PgExecutor<'_>,
        world_id: &str,
    ) -> AppResult<Vec<WorldVersion>> {
        sqlx::query_as!(
            WorldVersion,
            r#"
            SELECT * FROM world_versions
            WHERE world_id = $1
            ORDER BY created_at DESC
            "#,
            world_id,
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list world versions for '{}'", world_id))
        .map_err(Into::into)
    }

    /// Batch fetch latest version per world (for list endpoints)
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_latest(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<WorldVersion>> {
        let versions = sqlx::query_as!(
            WorldVersion,
            r#"
            SELECT DISTINCT ON (world_id) *
            FROM world_versions
            ORDER BY world_id, created_at DESC, id DESC
            "#,
        )
        .fetch_all(executor)
        .await
        .context("failed to batch fetch latest world versions")?;

        debug!(
            count = versions.len(),
            "Batch fetched latest world versions"
        );
        Ok(versions)
    }
}
