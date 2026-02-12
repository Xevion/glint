use anyhow::Context;
use serde_json::json;
use sqlx::PgPool;
use tracing::{debug, instrument};

use crate::error::AppResult;
use crate::extraction::ShaderPackData;
use crate::extraction::shader_props::{
    ParsedProfile, PipelineFeatures, ProfileOption, ScreenDefinition, ScreenEntry,
    ShaderPropertiesData,
};
use crate::models::extraction::{ShaderVersionMetadata, ShaderVersionProfile};

pub struct ExtractionRepo;

/// Generate a deterministic profile ID from (version_id, profile_name).
///
/// Matches the scheme used in migration 0023's backfill:
/// `hex(sha256(version_id || '/' || name))`
fn deterministic_profile_id(version_id: &str, profile_name: &str) -> String {
    use sha2::{Digest, Sha256};
    let input = format!("{version_id}/{profile_name}");
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(hash)
}

impl ExtractionRepo {
    /// Replace all profiles for a shader version with freshly extracted ones,
    /// and upsert the metadata sidecar. Runs in a single transaction.
    #[instrument(skip(pool, data), level = "debug")]
    pub async fn persist_extraction(
        pool: &PgPool,
        version_id: &str,
        data: &ShaderPackData,
    ) -> AppResult<()> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        // Delete existing profiles
        sqlx::query!(
            "DELETE FROM shader_version_profiles WHERE shader_version_id = $1",
            version_id
        )
        .execute(&mut *tx)
        .await
        .context(format!(
            "failed to delete existing profiles for version '{}'",
            version_id
        ))?;

        // Insert new profiles
        if let Some(props) = &data.properties {
            for profile in &props.profiles {
                let id = deterministic_profile_id(version_id, &profile.name);
                let label = data
                    .lang
                    .as_ref()
                    .and_then(|l| l.profile_labels.get(&profile.name).cloned());
                let options = serialize_profile_options(profile);

                sqlx::query!(
                    r#"
                    INSERT INTO shader_version_profiles (id, shader_version_id, name, label, options, sort_order)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    id,
                    version_id,
                    profile.name,
                    label,
                    options,
                    profile.sort_order,
                )
                .execute(&mut *tx)
                .await
                .context(format!(
                    "failed to insert profile '{}' for version '{}'",
                    profile.name, version_id
                ))?;
            }

            debug!(
                count = props.profiles.len(),
                version_id, "Inserted shader version profiles"
            );
        }

        // Upsert metadata
        let pipeline_features = data
            .properties
            .as_ref()
            .map(|p| serialize_pipeline_features(&p.pipeline));
        let iris_features_required = data
            .properties
            .as_ref()
            .map(|p| json!(p.iris_features_required));
        let iris_features_optional = data
            .properties
            .as_ref()
            .map(|p| json!(p.iris_features_optional));
        let settings_screen = data.properties.as_ref().map(serialize_screens);
        let file_paths = json!(data.scan.file_paths);
        let dimension_support = json!(data.scan.dimension_support);

        sqlx::query!(
            r#"
            INSERT INTO shader_version_metadata (
                shader_version_id, pipeline_features, iris_features_required,
                iris_features_optional, settings_screen, file_paths,
                dimension_support, has_custom_textures, extracted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            ON CONFLICT (shader_version_id) DO UPDATE SET
                pipeline_features = EXCLUDED.pipeline_features,
                iris_features_required = EXCLUDED.iris_features_required,
                iris_features_optional = EXCLUDED.iris_features_optional,
                settings_screen = EXCLUDED.settings_screen,
                file_paths = EXCLUDED.file_paths,
                dimension_support = EXCLUDED.dimension_support,
                has_custom_textures = EXCLUDED.has_custom_textures,
                extracted_at = EXCLUDED.extracted_at
            "#,
            version_id,
            pipeline_features,
            iris_features_required,
            iris_features_optional,
            settings_screen,
            file_paths,
            dimension_support,
            data.scan.has_custom_textures,
        )
        .execute(&mut *tx)
        .await
        .context(format!(
            "failed to upsert metadata for version '{}'",
            version_id
        ))?;

        // Mark extraction complete
        sqlx::query!(
            r#"
            UPDATE shader_versions
            SET extraction_status = 'completed',
                extraction_error = NULL,
                extracted_at = now()
            WHERE id = $1
            "#,
            version_id,
        )
        .execute(&mut *tx)
        .await
        .context(format!(
            "failed to mark extraction complete for version '{}'",
            version_id
        ))?;

        tx.commit()
            .await
            .context("failed to commit extraction transaction")?;

        debug!("Persisted extraction data");
        Ok(())
    }

    /// List all profiles for a shader version, ordered by sort_order.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_profiles_by_version(
        executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        version_id: &str,
    ) -> AppResult<Vec<ShaderVersionProfile>> {
        let profiles = sqlx::query_as!(
            ShaderVersionProfile,
            r#"
            SELECT id, shader_version_id, name, label, description, options, sort_order, created_at
            FROM shader_version_profiles
            WHERE shader_version_id = $1
            ORDER BY sort_order
            "#,
            version_id,
        )
        .fetch_all(executor)
        .await
        .context(format!(
            "failed to list profiles for version '{version_id}'"
        ))?;

        debug!(count = profiles.len(), "Listed profiles by version");
        Ok(profiles)
    }

    /// Get extracted metadata for a shader version, if available.
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_metadata_by_version(
        executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        version_id: &str,
    ) -> AppResult<Option<ShaderVersionMetadata>> {
        let metadata = sqlx::query_as!(
            ShaderVersionMetadata,
            r#"
            SELECT shader_version_id, pipeline_features, iris_features_required,
                   iris_features_optional, settings_screen, file_paths,
                   dimension_support, has_custom_textures, extracted_at
            FROM shader_version_metadata
            WHERE shader_version_id = $1
            "#,
            version_id,
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to get metadata for version '{version_id}'"))?;

        Ok(metadata)
    }
}

/// Serialize a profile's options as a JSON object: `{"KEY": "VALUE", "KEY2": "true", ...}`
fn serialize_profile_options(profile: &ParsedProfile) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for opt in &profile.options {
        match opt {
            ProfileOption::Enable(key) => {
                map.insert(key.clone(), json!("true"));
            }
            ProfileOption::Disable(key) => {
                map.insert(key.clone(), json!("false"));
            }
            ProfileOption::Set(key, value) => {
                map.insert(key.clone(), json!(value));
            }
        }
    }
    serde_json::Value::Object(map)
}

/// Serialize pipeline features as a JSON object: `{"separateAo": true, "clouds": "fancy", ...}`
fn serialize_pipeline_features(pipeline: &PipelineFeatures) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, val) in &pipeline.booleans {
        map.insert(key.clone(), json!(val));
    }
    if let Some(cloud) = &pipeline.cloud_setting {
        map.insert("clouds".to_string(), json!(cloud));
    }
    serde_json::Value::Object(map)
}

/// Serialize screen definitions as a JSON array of screen objects.
fn serialize_screens(data: &ShaderPropertiesData) -> serde_json::Value {
    let mut screens = Vec::new();

    if let Some(main) = &data.main_screen {
        screens.push(serialize_one_screen(main, true));
    }

    for screen in &data.screens {
        screens.push(serialize_one_screen(screen, false));
    }

    json!(screens)
}

fn serialize_one_screen(screen: &ScreenDefinition, is_main: bool) -> serde_json::Value {
    let entries: Vec<serde_json::Value> = screen
        .options
        .iter()
        .map(|entry| match entry {
            ScreenEntry::Option(name) => json!({"type": "option", "name": name}),
            ScreenEntry::SubScreen(name) => json!({"type": "screen", "name": name}),
            ScreenEntry::Empty => json!({"type": "empty"}),
        })
        .collect();

    let mut obj = serde_json::Map::new();
    obj.insert("name".to_string(), json!(screen.name));
    obj.insert("entries".to_string(), json!(entries));
    if is_main {
        obj.insert("is_main".to_string(), json!(true));
    }
    if let Some(cols) = screen.columns {
        obj.insert("columns".to_string(), json!(cols));
    }
    serde_json::Value::Object(obj)
}
