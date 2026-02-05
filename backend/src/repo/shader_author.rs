use anyhow::Context;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::ShaderAuthor;

pub struct ShaderAuthorRepo;

impl ShaderAuthorRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn list_by_shader(db: &DbPool, shader_id: &str) -> AppResult<Vec<ShaderAuthor>> {
        let authors = sqlx::query_as!(
            ShaderAuthor,
            "SELECT * FROM shader_authors WHERE shader_id = $1 ORDER BY name",
            shader_id
        )
        .fetch_all(db)
        .await
        .context(format!("failed to list authors for shader '{}'", shader_id))?;
        debug!(count = authors.len(), "Listed shader authors");
        Ok(authors)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn upsert(
        db: &DbPool,
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
        .fetch_one(db)
        .await
        .context(format!(
            "failed to upsert author '{}' for shader '{}'",
            name, shader_id
        ))
        .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn delete_by_shader(db: &DbPool, shader_id: &str) -> AppResult<u64> {
        let result = sqlx::query!("DELETE FROM shader_authors WHERE shader_id = $1", shader_id)
            .execute(db)
            .await
            .context(format!(
                "failed to delete authors for shader '{}'",
                shader_id
            ))?;
        Ok(result.rows_affected())
    }
}
