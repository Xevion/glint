use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::graphql::types::connection::CursorPage;
use crate::models::shader::AuthorAggregate;
use crate::models::{Shader, ShaderAuthor};
use crate::platform::PlatformAuthor;
use crate::slug::slugify;

/// ILIKE-safe search pattern from a raw query string.
fn search_pattern(q: &str) -> Option<String> {
    if q.is_empty() {
        return None;
    }
    Some(format!(
        "%{}%",
        q.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    ))
}

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

    /// List authors for multiple shaders in a single query, grouped by shader ID.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_shaders(
        executor: impl sqlx::PgExecutor<'_>,
        shader_ids: &[String],
    ) -> AppResult<HashMap<String, Vec<ShaderAuthor>>> {
        let authors = sqlx::query_as!(
            ShaderAuthor,
            "SELECT * FROM shader_authors WHERE shader_id = ANY($1) ORDER BY shader_id, name",
            shader_ids
        )
        .fetch_all(executor)
        .await
        .context("failed to list authors for shaders")?;

        debug!(count = authors.len(), "Listed shader authors (batch)");
        let mut map: HashMap<String, Vec<ShaderAuthor>> = HashMap::new();
        for author in authors {
            map.entry(author.shader_id.0.clone())
                .or_default()
                .push(author);
        }
        Ok(map)
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
        let slug = slugify(name);
        sqlx::query_as!(
            ShaderAuthor,
            r#"
            INSERT INTO shader_authors (id, shader_id, name, slug, url, platform)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (shader_id, name, platform) DO UPDATE SET
                url = COALESCE(EXCLUDED.url, shader_authors.url),
                slug = EXCLUDED.slug
            RETURNING *
            "#,
            id,
            shader_id,
            name,
            slug,
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
        let slugs: Vec<String> = authors.iter().map(|a| slugify(&a.name)).collect();
        let urls: Vec<Option<&str>> = authors.iter().map(|a| a.url.as_deref()).collect();
        let platforms: Vec<&str> = vec![platform; authors.len()];

        sqlx::query!(
            r#"
            INSERT INTO shader_authors (id, shader_id, name, slug, url, platform)
            SELECT unnest($1::text[]), unnest($2::text[]), unnest($3::text[]), unnest($4::text[]), unnest($5::text[]), unnest($6::text[])
            ON CONFLICT (shader_id, name, platform) DO UPDATE SET
                url = COALESCE(EXCLUDED.url, shader_authors.url),
                slug = EXCLUDED.slug
            "#,
            &ids as &[String],
            &shader_ids as &[&str],
            &names as &[&str],
            &slugs as &[String],
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

    /// Paginated list of unique authors aggregated across all shaders.
    /// Reads from the `author_aggregates` view (visibility filter built in).
    #[instrument(skip(db), level = "debug")]
    pub async fn list_authors_cursor(
        db: &DbPool,
        first: i32,
        after: Option<(i64, String)>,
        search: Option<&str>,
        sort: Option<&str>,
    ) -> AppResult<CursorPage<AuthorAggregate>> {
        use sqlx::QueryBuilder;

        let limit = first.clamp(1, 100) as i64;
        let pattern = search.and_then(search_pattern);

        let mut qb: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM author_aggregates WHERE TRUE");

        if let Some(ref p) = pattern {
            qb.push(" AND name ILIKE ");
            qb.push_bind(p.as_str());
        }

        if let Some((ref sort_val, ref cursor_name)) = after {
            let sort_key = sort.unwrap_or("popular");
            match sort_key {
                "name" => {
                    qb.push(" AND name > ");
                    qb.push_bind(cursor_name.as_str());
                }
                "shaders" => {
                    qb.push(" AND (shader_count, name) < (");
                    qb.push_bind(*sort_val);
                    qb.push(", ");
                    qb.push_bind(cursor_name.as_str());
                    qb.push(")");
                }
                _ => {
                    qb.push(" AND (total_views, name) < (");
                    qb.push_bind(*sort_val);
                    qb.push(", ");
                    qb.push_bind(cursor_name.as_str());
                    qb.push(")");
                }
            }
        }

        match sort.unwrap_or("popular") {
            "name" => qb.push(" ORDER BY name ASC"),
            "shaders" => qb.push(" ORDER BY shader_count DESC, name ASC"),
            _ => qb.push(" ORDER BY total_views DESC, name ASC"),
        };

        qb.push(" LIMIT ");
        qb.push_bind(limit + 1);

        let items: Vec<AuthorAggregate> = qb
            .build_query_as()
            .fetch_all(db)
            .await
            .context("failed to list authors (cursor)")?;

        let has_next_page = items.len() as i64 > limit;
        let items: Vec<AuthorAggregate> = items.into_iter().take(limit as usize).collect();

        // Count query
        let mut cqb: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM author_aggregates WHERE TRUE");

        if let Some(ref p) = pattern {
            cqb.push(" AND name ILIKE ");
            cqb.push_bind(p.as_str());
        }

        let total_count: i64 = cqb
            .build_query_scalar::<Option<i64>>()
            .fetch_one(db)
            .await
            .context("failed to count authors (cursor)")?
            .unwrap_or(0);

        debug!(total_count, has_next_page, "Listed authors (cursor)");
        Ok(CursorPage {
            items,
            has_next_page,
            has_previous_page: after.is_some(),
            total_count,
        })
    }

    /// Find an author by slug. Returns the aggregate from the `author_aggregates` view.
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug(db: &DbPool, slug: &str) -> AppResult<Option<AuthorAggregate>> {
        let row = sqlx::query_as!(
            AuthorAggregate,
            r#"
            SELECT
                name AS "name!",
                slug AS "slug!",
                shader_count AS "shader_count!",
                total_views AS "total_views!",
                total_captures AS "total_captures!",
                last_modified AS "last_modified!",
                image_path,
                thumbhash,
                top_shader_name,
                top_shader_slug
            FROM author_aggregates
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(db)
        .await
        .context("failed to find author by slug")?;

        Ok(row)
    }

    /// List shaders by a specific author name with cursor pagination.
    /// Uses `visible_shaders` view for the visibility filter.
    #[instrument(skip(db), level = "debug")]
    pub async fn list_shaders_by_author_cursor(
        db: &DbPool,
        author_name: &str,
        first: i32,
        after: Option<(DateTime<Utc>, String)>,
    ) -> AppResult<CursorPage<Shader>> {
        use sqlx::QueryBuilder;

        let limit = first.clamp(1, 100) as i64;

        let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "SELECT DISTINCT s.* FROM visible_shaders s
             JOIN shader_authors sa ON sa.shader_id = s.id
             WHERE sa.name = ",
        );
        qb.push_bind(author_name);

        if let Some((ref cursor_ts, ref cursor_id)) = after {
            qb.push(" AND (s.created_at, s.id) < (");
            qb.push_bind(*cursor_ts);
            qb.push(", ");
            qb.push_bind(cursor_id.as_str());
            qb.push(")");
        }

        qb.push(" ORDER BY s.view_count DESC, s.created_at DESC, s.id DESC");
        qb.push(" LIMIT ");
        qb.push_bind(limit + 1);

        let items: Vec<Shader> = qb
            .build_query_as()
            .fetch_all(db)
            .await
            .context("failed to list shaders by author (cursor)")?;

        let has_next_page = items.len() as i64 > limit;
        let items: Vec<Shader> = items.into_iter().take(limit as usize).collect();

        let total_count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT s.id) AS "count!"
            FROM visible_shaders s
            JOIN shader_authors sa ON sa.shader_id = s.id
            WHERE sa.name = $1
            "#,
            author_name
        )
        .fetch_one(db)
        .await
        .context("failed to count shaders by author")?;

        debug!(
            total_count,
            has_next_page, "Listed shaders by author (cursor)"
        );
        Ok(CursorPage {
            items,
            has_next_page,
            has_previous_page: after.is_some(),
            total_count,
        })
    }

    /// Get platform links for a specific author name.
    #[instrument(skip(db), level = "debug")]
    pub async fn list_platforms_by_author(
        db: &DbPool,
        author_name: &str,
    ) -> AppResult<Vec<(String, Option<String>)>> {
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT platform, url
            FROM shader_authors
            WHERE name = $1
            ORDER BY platform
            "#,
            author_name
        )
        .fetch_all(db as &DbPool)
        .await
        .context("failed to list platforms for author")?;

        Ok(rows.into_iter().map(|r| (r.platform, r.url)).collect())
    }
}
