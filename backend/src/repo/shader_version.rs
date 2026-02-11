use std::collections::HashMap;

use anyhow::Context;
use sqlx::types::Json;
use tracing::{debug, instrument};

use crate::error::{AppResult, OptionNotFoundExt};
use crate::id::{ShaderId, ShaderVersionId};
use crate::models::{CreateShaderVersionRequest, ShaderVersion, ShaderVersionDetail};
use crate::platform::PlatformVersion;

/// Intermediate row type for the version+count query (SELECT sv.*, COUNT(...))
#[derive(sqlx::FromRow)]
struct ShaderVersionWithCount {
    id: String,
    shader_id: String,
    version: String,
    modrinth_version_id: Option<String>,
    curseforge_file_id: Option<i32>,
    download_url: Option<String>,
    file_hash: Option<String>,
    file_size: Option<i64>,
    game_versions: Option<Json<Vec<String>>>,
    release_channel: Option<String>,
    supported_profiles: Option<Json<Vec<String>>>,
    upstream_published_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    capture_failure_count: i32,
    last_capture_error: Option<String>,
    capture_count: i64,
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
            r#"SELECT id, shader_id, version, modrinth_version_id, curseforge_file_id,
                download_url, file_hash, file_size,
                game_versions as "game_versions: Json<Vec<String>>",
                release_channel,
                supported_profiles as "supported_profiles: Json<Vec<String>>",
                upstream_published_at, created_at, capture_failure_count, last_capture_error
            FROM shader_versions WHERE shader_id = $1
            ORDER BY upstream_published_at DESC NULLS LAST, created_at DESC"#,
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

    /// List versions with per-version completed capture counts (for detail endpoints)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_shader_with_counts(
        executor: impl sqlx::PgExecutor<'_>,
        shader_id: &str,
    ) -> AppResult<Vec<ShaderVersionDetail>> {
        let rows = sqlx::query_as!(
            ShaderVersionWithCount,
            r#"
            SELECT
                sv.id, sv.shader_id, sv.version, sv.modrinth_version_id, sv.curseforge_file_id,
                sv.download_url, sv.file_hash, sv.file_size,
                sv.game_versions as "game_versions: Json<Vec<String>>",
                sv.release_channel,
                sv.supported_profiles as "supported_profiles: Json<Vec<String>>",
                sv.upstream_published_at, sv.created_at, sv.capture_failure_count, sv.last_capture_error,
                COUNT(c.id) FILTER (WHERE c.status = 'completed') as "capture_count!: i64"
            FROM shader_versions sv
            LEFT JOIN captures c ON c.shader_version_id = sv.id
            WHERE sv.shader_id = $1
            GROUP BY sv.id
            ORDER BY sv.upstream_published_at DESC NULLS LAST, sv.created_at DESC
            "#,
            shader_id
        )
        .fetch_all(executor)
        .await
        .context(format!(
            "failed to list versions with counts for shader '{}'",
            shader_id
        ))?;
        debug!(count = rows.len(), "Listed shader versions with counts");
        Ok(rows
            .into_iter()
            .map(|r| ShaderVersionDetail {
                version: ShaderVersion {
                    id: ShaderVersionId(r.id),
                    shader_id: ShaderId(r.shader_id),
                    version: r.version,
                    modrinth_version_id: r.modrinth_version_id,
                    curseforge_file_id: r.curseforge_file_id,
                    download_url: r.download_url,
                    file_hash: r.file_hash,
                    file_size: r.file_size,
                    game_versions: r.game_versions,
                    release_channel: r.release_channel,
                    supported_profiles: r.supported_profiles,
                    upstream_published_at: r.upstream_published_at,
                    created_at: r.created_at,
                    capture_failure_count: r.capture_failure_count,
                    last_capture_error: r.last_capture_error,
                },
                capture_count: r.capture_count,
            })
            .collect())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<ShaderVersion>> {
        sqlx::query_as!(
            ShaderVersion,
            r#"SELECT id, shader_id, version, modrinth_version_id, curseforge_file_id,
                download_url, file_hash, file_size,
                game_versions as "game_versions: Json<Vec<String>>",
                release_channel,
                supported_profiles as "supported_profiles: Json<Vec<String>>",
                upstream_published_at, created_at, capture_failure_count, last_capture_error
            FROM shader_versions WHERE id = $1"#,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to find shader version '{}'", id))
        .map_err(Into::into)
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
            RETURNING id, shader_id, version, modrinth_version_id, curseforge_file_id,
                download_url, file_hash, file_size,
                game_versions as "game_versions: Json<Vec<String>>",
                release_channel,
                supported_profiles as "supported_profiles: Json<Vec<String>>",
                upstream_published_at, created_at, capture_failure_count, last_capture_error
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

    #[instrument(skip(executor, profiles), level = "debug")]
    pub async fn update_supported_profiles(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        profiles: &[String],
    ) -> AppResult<()> {
        let json_value = Json(profiles.to_vec());
        sqlx::query!(
            "UPDATE shader_versions SET supported_profiles = $1 WHERE id = $2",
            json_value as Json<Vec<String>>,
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
    ) -> AppResult<HashMap<ShaderId, ShaderVersion>> {
        let versions = sqlx::query_as!(
            ShaderVersion,
            r#"
            SELECT DISTINCT ON (shader_id)
                id, shader_id, version, modrinth_version_id, curseforge_file_id,
                download_url, file_hash, file_size,
                game_versions as "game_versions: Json<Vec<String>>",
                release_channel,
                supported_profiles as "supported_profiles: Json<Vec<String>>",
                upstream_published_at, created_at, capture_failure_count, last_capture_error
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
    ) -> AppResult<ShaderId> {
        sqlx::query_scalar!(
            r#"SELECT shader_id AS "shader_id: ShaderId" FROM shader_versions WHERE id = $1"#,
            version_id
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to get shader_id for version '{}'",
            version_id
        ))?
        .or_not_found("Shader version", version_id)
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
        let version_id = crate::id::generate_id();
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
            version.game_versions.as_ref().map(|gv| Json(gv.clone())) as Option<Json<Vec<String>>>,
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

        // Deduplicate by version_number — platforms can return multiple entries for the
        // same version (e.g. different loaders), and PostgreSQL's ON CONFLICT DO UPDATE
        // cannot affect the same row twice in one statement.
        let mut seen = std::collections::HashSet::new();
        let versions: Vec<&PlatformVersion> = versions
            .iter()
            .filter(|v| seen.insert(&v.version_number))
            .collect();

        let ids: Vec<String> = versions.iter().map(|_| crate::id::generate_id()).collect();
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
        let game_versions: Vec<Option<Json<Vec<String>>>> = versions
            .iter()
            .map(|v| v.game_versions.as_ref().map(|gv| Json(gv.clone())))
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
                unnest($9::jsonb[]), unnest($10::text[]),
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
            &game_versions as &[Option<Json<Vec<String>>>],
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
            r#"SELECT id, shader_id, version, modrinth_version_id, curseforge_file_id,
                download_url, file_hash, file_size,
                game_versions as "game_versions: Json<Vec<String>>",
                release_channel,
                supported_profiles as "supported_profiles: Json<Vec<String>>",
                upstream_published_at, created_at, capture_failure_count, last_capture_error
            FROM shader_versions WHERE shader_id = $1
            ORDER BY upstream_published_at DESC NULLS LAST, created_at DESC LIMIT 1"#,
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
