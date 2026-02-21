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

    /// Paginated list of unique authors aggregated across all shaders.
    /// Only includes authors whose shaders have completed captures from active scenes.
    #[instrument(skip(db), level = "debug")]
    pub async fn list_authors_cursor(
        db: &DbPool,
        first: i32,
        after: Option<(DateTime<Utc>, String)>,
        search: Option<&str>,
        sort: Option<&str>,
    ) -> AppResult<CursorPage<AuthorAggregate>> {
        use sqlx::QueryBuilder;

        let limit = first.clamp(1, 100) as i64;

        let search_pattern = search.filter(|q| !q.is_empty()).map(|q| {
            format!(
                "%{}%",
                q.replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            )
        });

        let visibility_join = "
            JOIN shaders s ON s.id = sa.shader_id
            WHERE s.deleted_at IS NULL
              AND EXISTS (
                SELECT 1 FROM captures c
                JOIN shader_versions sv ON c.shader_version_id = sv.id
                JOIN scenes sc ON c.scene_id = sc.id
                WHERE sv.shader_id = s.id
                  AND c.status = 'completed'
                  AND c.image_path IS NOT NULL
                  AND sc.active = TRUE
            )";

        let base_select = "
            SELECT
                sa.name,
                COUNT(DISTINCT sa.shader_id)::BIGINT AS shader_count,
                COALESCE(SUM(s.view_count), 0)::BIGINT AS total_views,
                COALESCE((SELECT COUNT(DISTINCT c2.id)
                 FROM shader_authors sa2
                 JOIN shaders s2 ON s2.id = sa2.shader_id
                 JOIN shader_versions sv2 ON sv2.shader_id = s2.id
                 JOIN captures c2 ON c2.shader_version_id = sv2.id AND c2.status = 'completed'
                 WHERE sa2.name = sa.name AND s2.deleted_at IS NULL
                ), 0)::BIGINT AS total_captures,
                MAX(s.updated_at) AS last_modified,
                (SELECT c3.image_path FROM shaders s3
                 JOIN shader_authors sa3 ON sa3.shader_id = s3.id
                 JOIN shader_versions sv3 ON sv3.shader_id = s3.id
                 JOIN captures c3 ON c3.shader_version_id = sv3.id AND c3.status = 'completed' AND c3.image_path IS NOT NULL
                 WHERE sa3.name = sa.name AND s3.deleted_at IS NULL
                 ORDER BY s3.view_count DESC
                 LIMIT 1
                ) AS image_path,
                (SELECT c4.thumbhash FROM shaders s4
                 JOIN shader_authors sa4 ON sa4.shader_id = s4.id
                 JOIN shader_versions sv4 ON sv4.shader_id = s4.id
                 JOIN captures c4 ON c4.shader_version_id = sv4.id AND c4.status = 'completed' AND c4.image_path IS NOT NULL
                 WHERE sa4.name = sa.name AND s4.deleted_at IS NULL
                 ORDER BY s4.view_count DESC
                 LIMIT 1
                ) AS thumbhash,
                (SELECT s5.name FROM shaders s5
                 JOIN shader_authors sa5 ON sa5.shader_id = s5.id
                 WHERE sa5.name = sa.name AND s5.deleted_at IS NULL
                 ORDER BY s5.view_count DESC
                 LIMIT 1
                ) AS top_shader_name,
                (SELECT s6.slug FROM shaders s6
                 JOIN shader_authors sa6 ON sa6.shader_id = s6.id
                 WHERE sa6.name = sa.name AND s6.deleted_at IS NULL
                 ORDER BY s6.view_count DESC
                 LIMIT 1
                ) AS top_shader_slug
            FROM shader_authors sa";

        let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(base_select);
        qb.push(visibility_join);

        if let Some(ref pattern) = search_pattern {
            qb.push(" AND sa.name ILIKE ");
            qb.push_bind(pattern.as_str());
        }

        qb.push(" GROUP BY sa.name");

        // Cursor (keyset pagination over aggregated results via HAVING)
        if let Some((ref cursor_ts, ref cursor_name)) = after {
            let sort_value = sort.unwrap_or("popular");
            match sort_value {
                "name" => {
                    qb.push(" HAVING sa.name > ");
                    qb.push_bind(cursor_name.as_str());
                }
                "shaders" => {
                    qb.push(" HAVING (COUNT(DISTINCT sa.shader_id), sa.name) < (");
                    // Use cursor_ts as a proxy for the sort column value encoded in the cursor
                    // We encode shader_count in the timestamp field for this sort
                    qb.push_bind(cursor_ts.timestamp_millis());
                    qb.push(", ");
                    qb.push_bind(cursor_name.as_str());
                    qb.push(")");
                }
                _ => {
                    // popular: ORDER BY total_views DESC, name ASC
                    qb.push(" HAVING (COALESCE(SUM(s.view_count), 0), sa.name) < (");
                    qb.push_bind(cursor_ts.timestamp_millis());
                    qb.push(", ");
                    qb.push_bind(cursor_name.as_str());
                    qb.push(")");
                }
            }
        }

        let sort_value = sort.unwrap_or("popular");
        match sort_value {
            "name" => qb.push(" ORDER BY sa.name ASC"),
            "shaders" => qb.push(" ORDER BY COUNT(DISTINCT sa.shader_id) DESC, sa.name ASC"),
            _ => qb.push(" ORDER BY COALESCE(SUM(s.view_count), 0) DESC, sa.name ASC"),
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
            QueryBuilder::new("SELECT COUNT(DISTINCT sa.name) FROM shader_authors sa");
        cqb.push(visibility_join);

        if let Some(ref pattern) = search_pattern {
            cqb.push(" AND sa.name ILIKE ");
            cqb.push_bind(pattern.as_str());
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

    /// Find an author by slugified name. Scans all distinct author names and matches.
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug(db: &DbPool, slug: &str) -> AppResult<Option<AuthorAggregate>> {
        // Fetch all distinct author names and match by slug
        let names: Vec<String> = sqlx::query_scalar!(
            r#"SELECT DISTINCT name AS "name!" FROM shader_authors ORDER BY name"#
        )
        .fetch_all(db as &DbPool)
        .await
        .context("failed to fetch distinct author names")?;

        let matched_name = names.into_iter().find(|name| slugify(name) == slug);

        let Some(author_name) = matched_name else {
            return Ok(None);
        };

        // Fetch the full aggregate for this author
        let row = sqlx::query_as!(
            AuthorAggregate,
            r#"
            SELECT
                sa.name,
                COUNT(DISTINCT sa.shader_id) AS "shader_count!",
                COALESCE(SUM(s.view_count), 0)::BIGINT AS "total_views!",
                COALESCE((SELECT COUNT(DISTINCT c2.id)
                 FROM shader_authors sa2
                 JOIN shaders s2 ON s2.id = sa2.shader_id
                 JOIN shader_versions sv2 ON sv2.shader_id = s2.id
                 JOIN captures c2 ON c2.shader_version_id = sv2.id AND c2.status = 'completed'
                 WHERE sa2.name = sa.name AND s2.deleted_at IS NULL
                ), 0) AS "total_captures!",
                MAX(s.updated_at) AS "last_modified!",
                (SELECT c3.image_path FROM shaders s3
                 JOIN shader_authors sa3 ON sa3.shader_id = s3.id
                 JOIN shader_versions sv3 ON sv3.shader_id = s3.id
                 JOIN captures c3 ON c3.shader_version_id = sv3.id AND c3.status = 'completed' AND c3.image_path IS NOT NULL
                 WHERE sa3.name = sa.name AND s3.deleted_at IS NULL
                 ORDER BY s3.view_count DESC
                 LIMIT 1
                ) AS image_path,
                (SELECT c4.thumbhash FROM shaders s4
                 JOIN shader_authors sa4 ON sa4.shader_id = s4.id
                 JOIN shader_versions sv4 ON sv4.shader_id = s4.id
                 JOIN captures c4 ON c4.shader_version_id = sv4.id AND c4.status = 'completed' AND c4.image_path IS NOT NULL
                 WHERE sa4.name = sa.name AND s4.deleted_at IS NULL
                 ORDER BY s4.view_count DESC
                 LIMIT 1
                ) AS thumbhash,
                (SELECT s5.name FROM shaders s5
                 JOIN shader_authors sa5 ON sa5.shader_id = s5.id
                 WHERE sa5.name = sa.name AND s5.deleted_at IS NULL
                 ORDER BY s5.view_count DESC
                 LIMIT 1
                ) AS top_shader_name,
                (SELECT s6.slug FROM shaders s6
                 JOIN shader_authors sa6 ON sa6.shader_id = s6.id
                 WHERE sa6.name = sa.name AND s6.deleted_at IS NULL
                 ORDER BY s6.view_count DESC
                 LIMIT 1
                ) AS top_shader_slug
            FROM shader_authors sa
            JOIN shaders s ON s.id = sa.shader_id
            WHERE sa.name = $1 AND s.deleted_at IS NULL
            GROUP BY sa.name
            "#,
            &author_name
        )
        .fetch_optional(db)
        .await
        .context("failed to fetch author aggregate")?;

        Ok(row)
    }

    /// List shaders by a specific author name with cursor pagination.
    /// Only returns shaders with completed captures from active scenes.
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
            "SELECT DISTINCT s.* FROM shaders s
             JOIN shader_authors sa ON sa.shader_id = s.id
             WHERE s.deleted_at IS NULL AND sa.name = ",
        );
        qb.push_bind(author_name);

        qb.push(
            " AND EXISTS (
                SELECT 1 FROM captures c
                JOIN shader_versions sv ON c.shader_version_id = sv.id
                JOIN scenes sc ON c.scene_id = sc.id
                WHERE sv.shader_id = s.id
                  AND c.status = 'completed'
                  AND c.image_path IS NOT NULL
                  AND sc.active = TRUE
            )",
        );

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

        // Count query
        let total_count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT s.id) AS "count!"
            FROM shaders s
            JOIN shader_authors sa ON sa.shader_id = s.id
            WHERE s.deleted_at IS NULL
              AND sa.name = $1
              AND EXISTS (
                SELECT 1 FROM captures c
                JOIN shader_versions sv ON c.shader_version_id = sv.id
                JOIN scenes sc ON c.scene_id = sc.id
                WHERE sv.shader_id = s.id
                  AND c.status = 'completed'
                  AND c.image_path IS NOT NULL
                  AND sc.active = TRUE
              )
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
