use std::collections::HashMap;

use anyhow::Context;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{
    CaptureWithContext, CreateShaderRequest, CreateShaderVersionRequest, Shader, ShaderVersion,
    UpdateShaderRequest,
};
use crate::platform::{Platform, PlatformMetadata, PlatformVersion};

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

    /// Resolve a shader by UUID or slug. If the input parses as a UUID, looks up
    /// by ID; otherwise falls back to slug.
    #[instrument(skip(db), level = "debug")]
    pub async fn get(db: &DbPool, id_or_slug: &str) -> AppResult<Shader> {
        if Uuid::try_parse(id_or_slug).is_ok() {
            Self::get_by_id(db, id_or_slug).await
        } else {
            Self::get_by_slug(db, id_or_slug).await
        }
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
    pub async fn get_by_slug(executor: impl sqlx::PgExecutor<'_>, slug: &str) -> AppResult<Shader> {
        Self::find_by_slug(executor, slug)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Shader '{}' not found", slug)))
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
    pub async fn get_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<Shader> {
        Self::find_by_id(executor, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Shader with id '{}' not found", id)))
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
            req.slug,
            req.description,
            req.modrinth_id,
            req.curseforge_id,
            req.website_url
        )
        .fetch_one(executor)
        .await;

        match result {
            Ok(shader) => Ok(shader),
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => Err(
                AppError::Conflict(format!("Shader with slug '{}' already exists", req.slug)),
            ),
            Err(e) => Err(e).context(format!("failed to create shader '{}'", req.slug))?,
        }
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
                description = COALESCE($2, description),
                modrinth_id = COALESCE($3, modrinth_id),
                curseforge_id = COALESCE($4, curseforge_id),
                website_url = COALESCE($5, website_url),
                updated_at = now()
            WHERE id = $6
            RETURNING *
            "#,
            req.name,
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

        if let Err(sqlx::Error::Database(db_err)) = &result
            && db_err.code().as_deref() == Some("23505")
        {
            return Err(AppError::Conflict(format!(
                "Shader with slug '{}' already exists",
                metadata.slug
            )));
        }
        result.context(format!(
            "failed to create shader '{}' from platform",
            metadata.slug
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

    /// Fetch captures with shader/version context for a given shader
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_captures_with_context(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
    ) -> AppResult<Vec<CaptureWithContext>> {
        let captures = sqlx::query_as::<_, CaptureWithContext>(
            r#"
            SELECT
                c.id,
                c.scene_id,
                s.slug as shader_slug,
                s.name as shader_name,
                sv.version as shader_version,
                c.profile,
                c.screenshot_path,
                c.screenshot_url,
                c.captured_at,
                c.resolution_width,
                c.resolution_height,
                cri.run_id,
                cr.status as run_status
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            LEFT JOIN capture_run_items cri ON cri.capture_id = c.id
            LEFT JOIN capture_runs cr ON cri.run_id = cr.id
            WHERE s.id = $1 AND c.status = 'completed'
            ORDER BY sv.created_at DESC
            "#,
        )
        .bind(shader_id)
        .fetch_all(executor)
        .await
        .context(format!("failed to get captures for shader '{}'", shader_id))?;

        debug!(count = captures.len(), "Fetched captures for shader");
        Ok(captures)
    }
}

pub struct ShaderVersionRepo;

impl ShaderVersionRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_shader(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
    ) -> AppResult<Vec<ShaderVersion>> {
        let versions = sqlx::query_as!(
            ShaderVersion,
            "SELECT * FROM shader_versions WHERE shader_id = $1",
            shader_id
        )
        .fetch_all(executor)
        .await
        .context(format!(
            "failed to list versions for shader '{}'",
            shader_id
        ))?;
        debug!(count = versions.len(), "Listed shader versions");
        Ok(versions)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<ShaderVersion>> {
        sqlx::query_as!(
            ShaderVersion,
            "SELECT * FROM shader_versions WHERE id = $1",
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find shader version '{}'", id))
        .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<ShaderVersion> {
        Self::find_by_id(executor, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Shader version '{}' not found", id)))
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM shader_versions WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to check shader version existence '{}'", id))?;
        Ok(result.is_some())
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        shader_id: &str,
        req: &CreateShaderVersionRequest,
    ) -> AppResult<ShaderVersion> {
        sqlx::query_as!(
            ShaderVersion,
            r#"
            INSERT INTO shader_versions (id, shader_id, version, modrinth_version_id, download_url, file_hash, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, now())
            RETURNING *
            "#,
            id,
            shader_id,
            req.version,
            req.modrinth_version_id,
            req.download_url,
            req.file_hash
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to create shader version '{}'", req.version))
        .map_err(Into::into)
    }

    #[instrument(skip(executor, profiles_json), level = "debug")]
    pub async fn update_supported_profiles(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        profiles_json: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "UPDATE shader_versions SET supported_profiles = $1 WHERE id = $2",
            profiles_json,
            id
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to update supported profiles for version '{}'",
            id
        ))?;
        Ok(())
    }

    /// Get the parent shader's slug for a given shader version ID.
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_shader_slug(
        executor: impl sqlx::PgExecutor<'_>,
        version_id: &str,
    ) -> AppResult<Option<String>> {
        sqlx::query_scalar!(
            r#"
            SELECT s.slug FROM shaders s
            JOIN shader_versions sv ON sv.shader_id = s.id
            WHERE sv.id = $1
            "#,
            version_id
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to get shader slug for version '{}'",
            version_id
        ))
        .map_err(Into::into)
    }

    /// Get the latest version per shader in a single query
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_latest_versions(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<HashMap<String, ShaderVersion>> {
        let versions = sqlx::query_as!(
            ShaderVersion,
            r#"
            SELECT DISTINCT ON (shader_id) *
            FROM shader_versions
            ORDER BY shader_id, upstream_published_at DESC NULLS LAST, created_at DESC
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to batch fetch latest shader versions")?;

        debug!(count = versions.len(), "Batch fetched latest versions");
        Ok(versions
            .into_iter()
            .map(|v| (v.shader_id.clone(), v))
            .collect())
    }

    /// Get the parent shader's ID for a given shader version.
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_shader_id(
        executor: impl sqlx::PgExecutor<'_>,
        version_id: &str,
    ) -> AppResult<String> {
        sqlx::query_scalar!(
            "SELECT shader_id FROM shader_versions WHERE id = $1",
            version_id
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to get shader_id for version '{}'",
            version_id
        ))?
        .ok_or_else(|| AppError::NotFound(format!("Shader version '{}' not found", version_id)))
    }

    /// Increment the capture failure count and record the error message.
    #[instrument(skip(executor), level = "debug")]
    pub async fn increment_failure_count(
        executor: impl sqlx::PgExecutor<'_>,
        version_id: &str,
        error_message: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE shader_versions
            SET capture_failure_count = capture_failure_count + 1,
                last_capture_error = $2
            WHERE id = $1
            "#,
            version_id,
            error_message,
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to increment failure count for version '{}'",
            version_id
        ))?;
        Ok(())
    }

    /// Upsert a version from platform data (consolidates all platform-specific INSERT/ON CONFLICT)
    #[instrument(skip(executor, version), level = "debug")]
    pub async fn upsert_from_platform(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
        version: &PlatformVersion,
    ) -> AppResult<()> {
        let version_id = Uuid::new_v4().to_string();
        sqlx::query!(
            r#"
            INSERT INTO shader_versions (
                id, shader_id, version, modrinth_version_id, curseforge_file_id,
                download_url, file_hash, file_size, game_versions, release_channel,
                upstream_published_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (shader_id, version) DO UPDATE SET
                modrinth_version_id = COALESCE(EXCLUDED.modrinth_version_id, shader_versions.modrinth_version_id),
                curseforge_file_id = COALESCE(EXCLUDED.curseforge_file_id, shader_versions.curseforge_file_id),
                download_url = COALESCE(EXCLUDED.download_url, shader_versions.download_url),
                file_hash = COALESCE(EXCLUDED.file_hash, shader_versions.file_hash),
                file_size = COALESCE(EXCLUDED.file_size, shader_versions.file_size),
                game_versions = COALESCE(EXCLUDED.game_versions, shader_versions.game_versions),
                release_channel = COALESCE(EXCLUDED.release_channel, shader_versions.release_channel),
                upstream_published_at = COALESCE(EXCLUDED.upstream_published_at, shader_versions.upstream_published_at)
            "#,
            version_id,
            shader_id,
            version.version_number,
            version.modrinth_version_id,
            version.curseforge_file_id,
            version.download_url,
            version.file_hash,
            version.file_size,
            version.game_versions_json,
            version.release_channel,
            version.published_at,
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to upsert version '{}' for shader '{}'",
            version.version_number, shader_id
        ))?;
        Ok(())
    }

    /// Batch upsert versions from platform data in a single query
    #[instrument(skip(executor, versions), level = "debug")]
    pub async fn upsert_versions_batch(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
        versions: &[PlatformVersion],
    ) -> AppResult<()> {
        if versions.is_empty() {
            return Ok(());
        }

        let ids: Vec<String> = versions
            .iter()
            .map(|_| Uuid::new_v4().to_string())
            .collect();
        let shader_ids: Vec<&str> = vec![shader_id; versions.len()];
        let version_numbers: Vec<&str> =
            versions.iter().map(|v| v.version_number.as_str()).collect();
        let modrinth_ids: Vec<Option<&str>> = versions
            .iter()
            .map(|v| v.modrinth_version_id.as_deref())
            .collect();
        let cf_file_ids: Vec<Option<i32>> = versions.iter().map(|v| v.curseforge_file_id).collect();
        let download_urls: Vec<Option<&str>> =
            versions.iter().map(|v| v.download_url.as_deref()).collect();
        let file_hashes: Vec<Option<&str>> =
            versions.iter().map(|v| v.file_hash.as_deref()).collect();
        let file_sizes: Vec<Option<i64>> = versions.iter().map(|v| v.file_size).collect();
        let game_versions: Vec<Option<&str>> = versions
            .iter()
            .map(|v| v.game_versions_json.as_deref())
            .collect();
        let release_channels: Vec<Option<&str>> = versions
            .iter()
            .map(|v| v.release_channel.as_deref())
            .collect();
        let published_ats: Vec<Option<chrono::DateTime<chrono::Utc>>> =
            versions.iter().map(|v| v.published_at).collect();

        sqlx::query!(
            r#"
            INSERT INTO shader_versions (
                id, shader_id, version, modrinth_version_id, curseforge_file_id,
                download_url, file_hash, file_size, game_versions, release_channel,
                upstream_published_at, created_at
            )
            SELECT
                unnest($1::text[]), unnest($2::text[]), unnest($3::text[]),
                unnest($4::text[]), unnest($5::int4[]),
                unnest($6::text[]), unnest($7::text[]), unnest($8::int8[]),
                unnest($9::text[]), unnest($10::text[]),
                unnest($11::timestamptz[]), now()
            ON CONFLICT (shader_id, version) DO UPDATE SET
                modrinth_version_id = COALESCE(EXCLUDED.modrinth_version_id, shader_versions.modrinth_version_id),
                curseforge_file_id = COALESCE(EXCLUDED.curseforge_file_id, shader_versions.curseforge_file_id),
                download_url = COALESCE(EXCLUDED.download_url, shader_versions.download_url),
                file_hash = COALESCE(EXCLUDED.file_hash, shader_versions.file_hash),
                file_size = COALESCE(EXCLUDED.file_size, shader_versions.file_size),
                game_versions = COALESCE(EXCLUDED.game_versions, shader_versions.game_versions),
                release_channel = COALESCE(EXCLUDED.release_channel, shader_versions.release_channel),
                upstream_published_at = COALESCE(EXCLUDED.upstream_published_at, shader_versions.upstream_published_at)
            "#,
            &ids as &[String],
            &shader_ids as &[&str],
            &version_numbers as &[&str],
            &modrinth_ids as &[Option<&str>],
            &cf_file_ids as &[Option<i32>],
            &download_urls as &[Option<&str>],
            &file_hashes as &[Option<&str>],
            &file_sizes as &[Option<i64>],
            &game_versions as &[Option<&str>],
            &release_channels as &[Option<&str>],
            &published_ats as &[Option<chrono::DateTime<chrono::Utc>>],
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to batch upsert {} versions for shader '{}'",
            versions.len(),
            shader_id
        ))?;

        debug!(
            count = versions.len(),
            shader_id, "Batch upserted shader versions"
        );
        Ok(())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_latest_for_shader(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
    ) -> AppResult<Option<ShaderVersion>> {
        sqlx::query_as!(
            ShaderVersion,
            "SELECT * FROM shader_versions WHERE shader_id = $1 ORDER BY upstream_published_at DESC NULLS LAST, created_at DESC LIMIT 1",
            shader_id
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to get latest version for shader '{}'",
            shader_id
        ))
        .map_err(Into::into)
    }
}
