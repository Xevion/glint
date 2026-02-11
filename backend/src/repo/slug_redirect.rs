use anyhow::Context;
use tracing::instrument;

use crate::error::AppResult;

pub struct SlugRedirectRepo;

impl SlugRedirectRepo {
    /// Look up where an old slug redirects to. Returns the entity_id if found.
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_entity_id(
        executor: impl sqlx::PgExecutor<'_>,
        entity_type: &str,
        old_slug: &str,
    ) -> AppResult<Option<String>> {
        sqlx::query_scalar!(
            "SELECT entity_id FROM slug_redirects WHERE entity_type = $1 AND old_slug = $2",
            entity_type,
            old_slug
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to look up slug redirect for {} '{}'",
            entity_type, old_slug
        ))
        .map_err(Into::into)
    }

    /// Record a slug redirect (upsert — if old_slug already redirects, update the target).
    #[instrument(skip(executor), level = "debug")]
    pub async fn upsert(
        executor: impl sqlx::PgExecutor<'_>,
        entity_type: &str,
        old_slug: &str,
        entity_id: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO slug_redirects (entity_type, old_slug, entity_id, created_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (entity_type, old_slug) DO UPDATE SET entity_id = $3
            "#,
            entity_type,
            old_slug,
            entity_id
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to upsert slug redirect for {} '{}'",
            entity_type, old_slug
        ))?;
        Ok(())
    }

    /// Delete redirect entries pointing to an entity (cleanup when entity is deleted).
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete_for_entity(
        executor: impl sqlx::PgExecutor<'_>,
        entity_type: &str,
        entity_id: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "DELETE FROM slug_redirects WHERE entity_type = $1 AND entity_id = $2",
            entity_type,
            entity_id
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to delete slug redirects for {} '{}'",
            entity_type, entity_id
        ))?;
        Ok(())
    }

    /// Remove a specific old_slug redirect (e.g., when old_slug is reused by a new entity).
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete_by_old_slug(
        executor: impl sqlx::PgExecutor<'_>,
        entity_type: &str,
        old_slug: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "DELETE FROM slug_redirects WHERE entity_type = $1 AND old_slug = $2",
            entity_type,
            old_slug
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to delete slug redirect {} '{}'",
            entity_type, old_slug
        ))?;
        Ok(())
    }
}
