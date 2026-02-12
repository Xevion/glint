use std::collections::HashMap;

use anyhow::Context;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppResult, OptionNotFoundExt, SqlxResultExt};
use crate::id::ShaderId;
use crate::models::{CreateShaderRequest, Shader, ShaderAdopted, UpdateShaderRequest};
use crate::platform::{Platform, PlatformMetadata};

pub struct ShaderRepo;

impl ShaderRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn list(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<Shader>> {
        let shaders = sqlx::query_as!(Shader, "SELECT * FROM shaders ORDER BY name")
            .fetch_all(executor)
            .await
            .context("failed to list shaders")?;
        debug!(count = shaders.len(), "Listed shaders");
        Ok(shaders)
    }

    /// Paginated list of shaders that have at least one completed capture
    /// from an active scene (i.e. shaders with thumbnails).
    ///
    /// Uses `COUNT(*) OVER()` to compute the total in a single query.
    /// Returns `(shaders, total)`.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_with_captures(
        executor: impl sqlx::PgExecutor<'_>,
        limit: i64,
        offset: i64,
    ) -> AppResult<(Vec<Shader>, i64)> {
        use chrono::{DateTime, Utc};

        struct Row {
            id: ShaderId,
            name: String,
            slug: String,
            description: Option<String>,
            modrinth_id: Option<String>,
            curseforge_id: Option<String>,
            website_url: Option<String>,
            icon_url: Option<String>,
            source_url: Option<String>,
            license_id: Option<String>,
            upstream_downloads: Option<i64>,
            upstream_updated_at: Option<DateTime<Utc>>,
            last_synced_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
            view_count: i64,
            total: i64,
        }

        let rows = sqlx::query_as!(
            Row,
            r#"
            SELECT
                s.*,
                COUNT(*) OVER() AS "total!"
            FROM shaders s
            WHERE EXISTS (
                SELECT 1 FROM captures c
                JOIN shader_versions sv ON c.shader_version_id = sv.id
                JOIN scenes sc ON c.scene_id = sc.id
                WHERE sv.shader_id = s.id
                  AND c.status = 'completed'
                  AND c.image_url IS NOT NULL
                  AND sc.active = TRUE
            )
            ORDER BY s.name
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .context("failed to list shaders with captures")?;

        let total = rows.first().map_or(0, |r| r.total);
        let shaders = rows
            .into_iter()
            .map(|r| Shader {
                id: r.id,
                name: r.name,
                slug: r.slug,
                description: r.description,
                modrinth_id: r.modrinth_id,
                curseforge_id: r.curseforge_id,
                website_url: r.website_url,
                icon_url: r.icon_url,
                source_url: r.source_url,
                license_id: r.license_id,
                upstream_downloads: r.upstream_downloads,
                upstream_updated_at: r.upstream_updated_at,
                last_synced_at: r.last_synced_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
                view_count: r.view_count,
            })
            .collect();

        debug!(total, "Listed shaders with captures (paginated)");
        Ok((shaders, total))
    }

    /// Resolve a shader by ID or slug. Inputs longer than `ID_LENGTH` can only
    /// be slugs; shorter ones could be either, so we check both in one query.
    #[instrument(skip(db), level = "debug")]
    pub async fn get(db: &DbPool, id_or_slug: &str) -> AppResult<Shader> {
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
        let shaders = sqlx::query_as!(Shader, "SELECT * FROM shaders WHERE id = ANY($1)", ids)
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
        let result = sqlx::query_scalar!("SELECT 1 as one FROM shaders WHERE slug = $1", slug)
            .fetch_optional(executor)
            .await
            .context(format!("failed to check shader existence '{}'", slug))?;
        Ok(result.is_some())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM shaders WHERE id = $1", id)
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

    #[instrument(skip(executor), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM shaders WHERE id = $1", id)
            .execute(executor)
            .await
            .context(format!("failed to delete shader '{}'", id))?;
        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn update(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        req: &UpdateShaderRequest,
    ) -> AppResult<Shader> {
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
            id
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
            WHERE modrinth_id = ANY($1) OR curseforge_id = ANY($2)
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
