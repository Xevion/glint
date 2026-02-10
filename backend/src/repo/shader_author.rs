use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::AppResult;
use crate::models::ShaderAuthor;
use crate::platform::PlatformAuthor;

pub struct ShaderAuthorRepo;

impl ShaderAuthorRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_shader(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
    ) -> AppResult<Vec<ShaderAuthor>> {
        let authors = sqlx::query_as!(
            ShaderAuthor,
            "SELECT * FROM shader_authors WHERE shader_id = $1 ORDER BY name",
            shader_id
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list authors for shader '{}'", shader_id))?;
        debug!(count = authors.len(), "Listed shader authors");
        Ok(authors)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn list_all(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<ShaderAuthor>> {
        let authors = sqlx::query_as!(
            ShaderAuthor,
            "SELECT * FROM shader_authors ORDER BY shader_id, name"
        )
        .fetch_all(executor)
        .await
        .context("failed to list all shader authors")?;
        debug!(count = authors.len(), "Listed all shader authors");
        Ok(authors)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn upsert(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        shader_id: &str,
        name: &str,
        url: Option<&str>,
        platform: &str,
    ) -> AppResult<ShaderAuthor> {
        sqlx::query_as!(
            ShaderAuthor,
            r#"
            INSERT INTO shader_authors (id, shader_id, name, url, platform)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (shader_id, name, platform) DO UPDATE SET
                url = COALESCE(EXCLUDED.url, shader_authors.url)
            RETURNING *
            "#,
            id,
            shader_id,
            name,
            url,
            platform
        )
        .fetch_one(executor)
        .await
        .context(format!(
            "failed to upsert author '{}' for shader '{}'",
            name, shader_id
        ))
        .map_err(Into::into)
    }

    /// Batch upsert authors from platform data in a single query
    #[instrument(skip(executor, authors), level = "debug")]
    pub async fn upsert_batch(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
        authors: &[PlatformAuthor],
        platform: &str,
    ) -> AppResult<()> {
        if authors.is_empty() {
            return Ok(());
        }

        let ids: Vec<String> = authors.iter().map(|_| crate::id::generate_id()).collect();
        let shader_ids: Vec<&str> = vec![shader_id; authors.len()];
        let names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
        let urls: Vec<Option<&str>> = authors.iter().map(|a| a.url.as_deref()).collect();
        let platforms: Vec<&str> = vec![platform; authors.len()];

        sqlx::query!(
            r#"
            INSERT INTO shader_authors (id, shader_id, name, url, platform)
            SELECT unnest($1::text[]), unnest($2::text[]), unnest($3::text[]), unnest($4::text[]), unnest($5::text[])
            ON CONFLICT (shader_id, name, platform) DO UPDATE SET
                url = COALESCE(EXCLUDED.url, shader_authors.url)
            "#,
            &ids as &[String],
            &shader_ids as &[&str],
            &names as &[&str],
            &urls as &[Option<&str>],
            &platforms as &[&str],
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to batch upsert {} authors for shader '{}'",
            authors.len(),
            shader_id
        ))?;

        debug!(
            count = authors.len(),
            shader_id, "Batch upserted shader authors"
        );
        Ok(())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn delete_by_shader(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
    ) -> AppResult<u64> {
        let result = sqlx::query!("DELETE FROM shader_authors WHERE shader_id = $1", shader_id)
            .execute(executor)
            .await
            .context(format!(
                "failed to delete authors for shader '{}'",
                shader_id
            ))?;
        Ok(result.rows_affected())
    }
}
