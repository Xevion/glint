use chrono::{DateTime, Utc};
use tracing::{debug, instrument};

use crate::error::AppResult;

/// Sitemap entry with a slug and last-modified timestamp.
pub struct SitemapEntry {
    pub slug: String,
    pub last_modified: DateTime<Utc>,
}

/// Author sitemap entry with a name and last-modified timestamp.
pub struct AuthorEntry {
    pub name: String,
    pub last_modified: DateTime<Utc>,
}

pub struct SitemapRepo;

impl SitemapRepo {
    /// Fetch all shader slugs with the most recent content change timestamp.
    ///
    /// Uses `GREATEST(shader.updated_at, latest_completed_capture.created_at)`
    /// so the timestamp reflects whichever happened more recently: a metadata
    /// edit or a new capture landing.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_shader_entries(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<SitemapEntry>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                s.slug,
                GREATEST(s.updated_at, COALESCE(MAX(c.created_at), s.updated_at))
                    AS "last_modified!"
            FROM shaders s
            LEFT JOIN shader_versions sv ON sv.shader_id = s.id
            LEFT JOIN captures c ON c.shader_version_id = sv.id AND c.status = 'completed'
            GROUP BY s.id, s.slug, s.updated_at
            ORDER BY s.slug
            "#
        )
        .fetch_all(executor)
        .await?;

        debug!(count = rows.len(), "Listed shader sitemap entries");
        Ok(rows
            .into_iter()
            .map(|r| SitemapEntry {
                slug: r.slug,
                last_modified: r.last_modified,
            })
            .collect())
    }

    /// Fetch all active scene slugs with their creation timestamp.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_scene_entries(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<SitemapEntry>> {
        let rows =
            sqlx::query!("SELECT slug, created_at FROM scenes WHERE active = TRUE ORDER BY slug")
                .fetch_all(executor)
                .await?;

        debug!(count = rows.len(), "Listed scene sitemap entries");
        Ok(rows
            .into_iter()
            .map(|r| SitemapEntry {
                slug: r.slug,
                last_modified: r.created_at,
            })
            .collect())
    }

    /// Fetch unique author names with the most recent shader update timestamp.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_author_entries(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<AuthorEntry>> {
        let rows = sqlx::query!(
            r#"
            SELECT sa.name, MAX(s.updated_at) AS "last_modified!"
            FROM shader_authors sa
            JOIN shaders s ON s.id = sa.shader_id
            GROUP BY sa.name
            ORDER BY sa.name
            "#
        )
        .fetch_all(executor)
        .await?;

        debug!(count = rows.len(), "Listed author sitemap entries");
        Ok(rows
            .into_iter()
            .map(|r| AuthorEntry {
                name: r.name,
                last_modified: r.last_modified,
            })
            .collect())
    }
}
