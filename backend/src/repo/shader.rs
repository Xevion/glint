use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use tracing::{debug, instrument, warn};

use crate::db::DbPool;
use crate::error::{AppResult, OptionNotFoundExt, SqlxResultExt};
use crate::graphql::types::common::Visibility;
use crate::graphql::types::connection::CursorPage;
use crate::id::ShaderId;
use crate::models::{CreateShaderRequest, Page, Shader, ShaderAdopted, UpdateShaderRequest};
use crate::platform::{Platform, PlatformMetadata};

pub struct ShaderRepo;

impl ShaderRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn list(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<Shader>> {
        let shaders = sqlx::query_as!(
            Shader,
            "SELECT * FROM shaders WHERE deleted_at IS NULL ORDER BY name"
        )
        .fetch_all(executor)
        .await
        .context("failed to list shaders")?;
        debug!(count = shaders.len(), "Listed shaders");
        Ok(shaders)
    }

    /// Paginated list of shaders with optional filtering to only those that
    /// have at least one completed capture from an active scene.
    ///
    /// When `require_captures` is `true`, only shaders with thumbnails are
    /// returned (public listing). When `false`, all shaders are returned
    /// (admin listing).
    ///
    /// Uses `COUNT(*) OVER()` to compute the total in a single query.
    /// Supports optional name search (ILIKE) and sort order.
    /// Returns `(shaders, total)`.
    #[instrument(skip(db, page), level = "debug")]
    pub async fn list_paginated(
        db: &DbPool,
        page: &Page,
        search: Option<&str>,
        sort: Option<&str>,
        require_captures: bool,
    ) -> AppResult<(Vec<Shader>, i64)> {
        use sqlx::QueryBuilder;

        let search_pattern = search.filter(|q| !q.is_empty()).map(|q| {
            format!(
                "%{}%",
                q.replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            )
        });

        let source = if require_captures {
            "visible_shaders"
        } else {
            "shaders"
        };
        let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(format!(
            "SELECT s.*, COUNT(*) OVER() AS total FROM {source} s WHERE s.deleted_at IS NULL"
        ));

        if let Some(ref pattern) = search_pattern {
            qb.push(" AND s.name ILIKE ");
            qb.push_bind(pattern.as_str());
        }

        let sort_value = sort.unwrap_or("popular");
        match sort_value {
            "name" => qb.push(" ORDER BY s.name ASC"),
            "updated" => qb.push(" ORDER BY s.updated_at DESC"),
            "popular" => {
                qb.push(" ORDER BY s.view_count DESC, s.upstream_downloads DESC NULLS LAST")
            }
            other => {
                warn!(sort = other, "Unknown sort value, defaulting to popular");
                qb.push(" ORDER BY s.view_count DESC, s.upstream_downloads DESC NULLS LAST")
            }
        };

        qb.push(" LIMIT ");
        qb.push_bind(page.limit_i64());
        qb.push(" OFFSET ");
        qb.push_bind(page.offset_i64());

        #[derive(sqlx::FromRow)]
        struct Row {
            #[sqlx(flatten)]
            shader: Shader,
            total: i64,
        }

        let rows: Vec<Row> = qb
            .build_query_as()
            .fetch_all(db)
            .await
            .context("failed to list shaders with captures")?;

        let total = rows.first().map_or(0, |r| r.total);
        let shaders = rows.into_iter().map(|r| r.shader).collect();

        debug!(total, "Listed shaders with captures (paginated)");
        Ok((shaders, total))
    }

    /// Cursor-based paginated list of shaders.
    ///
    /// Visibility controls which shaders are included:
    /// - `Exclude` (default): only shaders with completed captures from active scenes
    /// - `Include`: all shaders regardless of capture status
    /// - `Only`: only shaders WITHOUT completed captures (hidden/uncaptured)
    #[instrument(skip(db), level = "debug")]
    pub async fn list_cursor(
        db: &DbPool,
        first: i32,
        after: Option<(DateTime<Utc>, String)>,
        search: Option<&str>,
        sort: Option<&str>,
        visibility: Visibility,
    ) -> AppResult<CursorPage<Shader>> {
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

        let mut qb: QueryBuilder<'_, sqlx::Postgres> = match visibility {
            Visibility::Exclude => {
                QueryBuilder::new("SELECT s.* FROM visible_shaders s WHERE TRUE")
            }
            Visibility::Include => {
                QueryBuilder::new("SELECT s.* FROM shaders s WHERE s.deleted_at IS NULL")
            }
            Visibility::Only => QueryBuilder::new(
                "SELECT s.* FROM shaders s WHERE s.deleted_at IS NULL AND s.id NOT IN (SELECT id FROM visible_shaders)",
            ),
        };

        if let Some(ref pattern) = search_pattern {
            qb.push(" AND s.name ILIKE ");
            qb.push_bind(pattern.as_str());
        }

        if let Some((ref cursor_ts, ref cursor_id)) = after {
            qb.push(" AND (s.created_at, s.id) < (");
            qb.push_bind(*cursor_ts);
            qb.push(", ");
            qb.push_bind(cursor_id.as_str());
            qb.push(")");
        }

        let sort_value = sort.unwrap_or("created");
        match sort_value {
            "popular" => {
                qb.push(" ORDER BY s.view_count DESC, s.upstream_downloads DESC NULLS LAST, s.created_at DESC, s.id DESC")
            }
            "name" => qb.push(" ORDER BY s.name ASC, s.created_at DESC, s.id DESC"),
            _ => qb.push(" ORDER BY s.created_at DESC, s.id DESC"),
        };

        qb.push(" LIMIT ");
        qb.push_bind(limit + 1);

        let items: Vec<Shader> = qb
            .build_query_as()
            .fetch_all(db)
            .await
            .context("failed to list shaders (cursor)")?;

        let has_next_page = items.len() as i64 > limit;
        let items: Vec<Shader> = items.into_iter().take(limit as usize).collect();

        let mut cqb: QueryBuilder<'_, sqlx::Postgres> = match visibility {
            Visibility::Exclude => {
                QueryBuilder::new("SELECT COUNT(*) FROM visible_shaders s WHERE TRUE")
            }
            Visibility::Include => {
                QueryBuilder::new("SELECT COUNT(*) FROM shaders s WHERE s.deleted_at IS NULL")
            }
            Visibility::Only => QueryBuilder::new(
                "SELECT COUNT(*) FROM shaders s WHERE s.deleted_at IS NULL AND s.id NOT IN (SELECT id FROM visible_shaders)",
            ),
        };

        if let Some(ref pattern) = search_pattern {
            cqb.push(" AND s.name ILIKE ");
            cqb.push_bind(pattern.as_str());
        }

        let total_count: i64 = cqb
            .build_query_scalar::<Option<i64>>()
            .fetch_one(db)
            .await
            .context("failed to count shaders (cursor)")?
            .unwrap_or(0);

        debug!(total_count, has_next_page, "Listed shaders (cursor)");
        Ok(CursorPage {
            items,
            has_next_page,
            has_previous_page: after.is_some(),
            total_count,
        })
    }

    /// Resolve a shader by ID or slug. Inputs longer than `ID_LENGTH` can only
    /// be slugs; shorter ones could be either, so we check both in one query.
    ///
    /// When `include_deleted` is `true`, soft-deleted shaders are included
    /// (used by admin mutations that need to operate on deleted shaders).
    #[instrument(skip(db), level = "debug")]
    pub async fn get(db: &DbPool, id_or_slug: &str, include_deleted: bool) -> AppResult<Shader> {
        if include_deleted {
            if id_or_slug.len() <= crate::id::ID_LENGTH {
                sqlx::query_as!(
                    Shader,
                    "SELECT * FROM shaders WHERE id = $1 OR slug = $1",
                    id_or_slug
                )
                .fetch_optional(db)
                .await
                .context(format!("failed to find shader '{}'", id_or_slug))?
                .or_not_found("Shader", id_or_slug)
            } else {
                Self::find_by_slug_including_deleted(db, id_or_slug)
                    .await?
                    .or_not_found("Shader", id_or_slug)
            }
        } else if id_or_slug.len() <= crate::id::ID_LENGTH {
            sqlx::query_as!(
                Shader,
                "SELECT * FROM shaders WHERE (id = $1 OR slug = $1) AND deleted_at IS NULL",
                id_or_slug
            )
            .fetch_optional(db)
            .await
            .context(format!("failed to find shader '{}'", id_or_slug))?
            .or_not_found("Shader", id_or_slug)
        } else {
            Self::find_by_slug(db, id_or_slug)
                .await?
                .or_not_found("Shader", id_or_slug)
        }
    }

    /// Fetch multiple shaders by ID in a single query.
    /// Returns a map from shader ID to Shader.
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_many(
        executor: impl sqlx::PgExecutor<'_>,
        ids: &[String],
    ) -> AppResult<HashMap<String, Shader>> {
        let shaders = sqlx::query_as!(
            Shader,
            "SELECT * FROM shaders WHERE id = ANY($1) AND deleted_at IS NULL",
            ids
        )
        .fetch_all(executor)
        .await
        .context("failed to batch fetch shaders")?;

        debug!(count = shaders.len(), "Batch fetched shaders");
        Ok(shaders.into_iter().map(|s| (s.id.0.clone(), s)).collect())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<Option<Shader>> {
        sqlx::query_as!(
            Shader,
            "SELECT * FROM shaders WHERE slug = $1 AND deleted_at IS NULL",
            slug
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find shader '{}'", slug))
        .map_err(Into::into)
    }

    /// Find a shader by slug regardless of soft-delete status (admin use).
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_slug_including_deleted(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<Option<Shader>> {
        sqlx::query_as!(Shader, "SELECT * FROM shaders WHERE slug = $1", slug)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find shader '{}'", slug))
            .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<Shader>> {
        sqlx::query_as!(
            Shader,
            "SELECT * FROM shaders WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find shader by id '{}'", id))
        .map_err(Into::into)
    }

    /// Find a shader by ID regardless of soft-delete status (admin use).
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id_including_deleted(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<Shader>> {
        sqlx::query_as!(Shader, "SELECT * FROM shaders WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find shader by id '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query_scalar!(
            "SELECT 1 as one FROM shaders WHERE slug = $1 AND deleted_at IS NULL",
            slug
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to check shader existence '{}'", slug))?;
        Ok(result.is_some())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!(
            "SELECT 1 as one FROM shaders WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to check shader existence by id '{}'", id))?;
        Ok(result.is_some())
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        slug: &str,
        req: &CreateShaderRequest,
    ) -> AppResult<Shader> {
        let result = sqlx::query_as!(
            Shader,
            r#"
            INSERT INTO shaders (id, name, slug, description, modrinth_id, curseforge_id, website_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
            RETURNING *
            "#,
            id,
            req.name,
            slug,
            req.description,
            req.modrinth_id,
            req.curseforge_id,
            req.website_url
        )
        .fetch_one(executor)
        .await;

        result.conflict_on_unique(format!("Shader with slug '{}' already exists", slug))
    }

    /// Soft-delete a shader by setting `deleted_at`. Returns `true` if a row was updated.
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE shaders SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(executor)
        .await
        .context(format!("failed to soft-delete shader '{}'", id))?;
        Ok(result.rows_affected() > 0)
    }

    /// Restore a soft-deleted shader by clearing `deleted_at`.
    #[instrument(skip(executor), level = "debug")]
    pub async fn restore(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<Shader>> {
        sqlx::query_as!(
            Shader,
            "UPDATE shaders SET deleted_at = NULL WHERE id = $1 AND deleted_at IS NOT NULL RETURNING *",
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to restore shader '{}'", id))
        .map_err(Into::into)
    }

    /// Permanently delete a soft-deleted shader (hard delete with CASCADE).
    #[instrument(skip(executor), level = "debug")]
    pub async fn purge(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM shaders WHERE id = $1 AND deleted_at IS NOT NULL",
            id
        )
        .execute(executor)
        .await
        .context(format!("failed to purge shader '{}'", id))?;
        Ok(result.rows_affected() > 0)
    }

    /// List all soft-deleted shaders, most recently deleted first.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_deleted(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<Shader>> {
        let shaders = sqlx::query_as!(
            Shader,
            "SELECT * FROM shaders WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC"
        )
        .fetch_all(executor)
        .await
        .context("failed to list deleted shaders")?;
        debug!(count = shaders.len(), "Listed deleted shaders");
        Ok(shaders)
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn update(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        req: &UpdateShaderRequest,
    ) -> AppResult<Shader> {
        // Resolve preferred_version_id: None = don't change, Some("") = clear, Some(id) = set
        let preferred_version_update: Option<Option<String>> = req
            .preferred_version_id
            .as_ref()
            .map(|v| if v.is_empty() { None } else { Some(v.clone()) });
        let update_preferred = preferred_version_update.is_some();
        let preferred_value = preferred_version_update.flatten();

        sqlx::query_as!(
            Shader,
            r#"
            UPDATE shaders SET
                name = COALESCE($1, name),
                slug = COALESCE($2, slug),
                description = COALESCE($3, description),
                modrinth_id = COALESCE($4, modrinth_id),
                curseforge_id = COALESCE($5, curseforge_id),
                website_url = COALESCE($6, website_url),
                preferred_version_id = CASE WHEN $8 THEN $9 ELSE preferred_version_id END,
                capture_enabled = COALESCE($10, capture_enabled),
                updated_at = now()
            WHERE id = $7
            RETURNING *
            "#,
            req.name,
            req.slug,
            req.description,
            req.modrinth_id,
            req.curseforge_id,
            req.website_url,
            id,
            update_preferred,
            preferred_value,
            req.capture_enabled,
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to update shader '{}'", id))
        .map_err(Into::into)
    }

    /// Create a shader from platform metadata
    #[instrument(skip(executor, metadata), level = "debug")]
    pub async fn create_from_platform(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        platform: Platform,
        metadata: &PlatformMetadata,
    ) -> AppResult<()> {
        let (modrinth_id, curseforge_id) = match platform {
            Platform::Modrinth => (Some(metadata.platform_id.as_str()), None),
            Platform::CurseForge => (None, Some(metadata.platform_id.as_str())),
        };
        let now = chrono::Utc::now();

        let result = sqlx::query!(
            r#"
            INSERT INTO shaders (
                id, name, slug, description, modrinth_id, curseforge_id,
                website_url, icon_url, source_url, license_id,
                upstream_downloads, upstream_updated_at, last_synced_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14)
            "#,
            id,
            metadata.name,
            metadata.slug,
            metadata.description.as_deref(),
            modrinth_id,
            curseforge_id,
            metadata.website_url.as_deref(),
            metadata.icon_url.as_deref(),
            metadata.source_url.as_deref(),
            metadata.license_id.as_deref(),
            metadata.downloads as i64,
            metadata.updated_at,
            now,
            now,
        )
        .execute(executor)
        .await;

        result.conflict_on_unique(format!(
            "Shader with slug '{}' already exists",
            metadata.slug,
        ))?;
        Ok(())
    }

    /// Link a platform ID to an existing shader
    #[instrument(skip(executor), level = "debug")]
    pub async fn link_platform(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
        platform: Platform,
        platform_id: &str,
    ) -> AppResult<()> {
        match platform {
            Platform::Modrinth => {
                sqlx::query!(
                    "UPDATE shaders SET modrinth_id = $1, updated_at = now() WHERE id = $2",
                    platform_id,
                    shader_id
                )
                .execute(executor)
                .await
                .context("failed to link modrinth")?;
            }
            Platform::CurseForge => {
                sqlx::query!(
                    "UPDATE shaders SET curseforge_id = $1, updated_at = now() WHERE id = $2",
                    platform_id,
                    shader_id
                )
                .execute(executor)
                .await
                .context("failed to link curseforge")?;
            }
        }
        Ok(())
    }

    /// Update shader metadata during sync
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(executor), level = "debug")]
    pub async fn update_sync_metadata(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
        name: &str,
        description: Option<&str>,
        icon_url: Option<&str>,
        source_url: Option<&str>,
        license_id: Option<&str>,
        total_downloads: i64,
        upstream_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AppResult<()> {
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"
            UPDATE shaders SET
                name = $1,
                description = $2,
                icon_url = $3,
                source_url = $4,
                license_id = $5,
                upstream_downloads = $6,
                upstream_updated_at = $7,
                last_synced_at = $8,
                updated_at = $8
            WHERE id = $9
            "#,
            name,
            description,
            icon_url,
            source_url,
            license_id,
            total_downloads,
            upstream_updated_at,
            now,
            shader_id,
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to update sync metadata for shader '{}'",
            shader_id
        ))?;
        Ok(())
    }

    /// Find adopted shaders matching any of the given platform IDs.
    /// Returns a map from platform_id → ShaderAdopted for quick lookup.
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_adopted_by_platform_ids(
        executor: impl sqlx::PgExecutor<'_>,
        modrinth_ids: &[String],
        curseforge_ids: &[String],
    ) -> AppResult<HashMap<String, ShaderAdopted>> {
        struct Row {
            id: String,
            slug: String,
            modrinth_id: Option<String>,
            curseforge_id: Option<String>,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT id, slug, modrinth_id, curseforge_id FROM shaders
            WHERE deleted_at IS NULL AND (modrinth_id = ANY($1) OR curseforge_id = ANY($2))
            "#,
            modrinth_ids,
            curseforge_ids,
        )
        .fetch_all(executor)
        .await
        .context("failed to find adopted shaders by platform IDs")?;

        let mut result = HashMap::new();
        for row in rows {
            let adopted = ShaderAdopted {
                id: ShaderId(row.id),
                slug: row.slug,
            };
            if let Some(mid) = row.modrinth_id {
                result.insert(mid, adopted.clone());
            }
            if let Some(cid) = row.curseforge_id {
                result.insert(cid, adopted);
            }
        }

        debug!(
            count = result.len(),
            "Found adopted shaders by platform IDs"
        );
        Ok(result)
    }
}
