use std::collections::HashMap;

use assert2::{assert, check, let_assert};
use glint::db::apply_views;
use glint::extraction::ShaderPackData;
use glint::extraction::archive::ZipScanResult;
use glint::extraction::lang::LangData;
use glint::extraction::shader_props::{
    ParsedProfile, PipelineFeatures, ProfileOption, ShaderPropertiesData,
};
use glint::repo::ExtractionRepo;
use sha2::{Digest, Sha256};

mod helpers;
use helpers::*;

fn make_shader_pack_data(profiles: Vec<ParsedProfile>, lang: Option<LangData>) -> ShaderPackData {
    let properties = if profiles.is_empty() {
        None
    } else {
        Some(ShaderPropertiesData {
            profiles,
            screens: vec![],
            main_screen: None,
            iris_features_required: vec![],
            iris_features_optional: vec![],
            pipeline: PipelineFeatures {
                booleans: HashMap::new(),
                cloud_setting: None,
            },
            slider_options: vec![],
        })
    };
    ShaderPackData {
        properties,
        lang,
        scan: ZipScanResult {
            file_paths: vec!["shaders/gbuffers_terrain.vsh".into()],
            dimension_support: vec!["overworld".into()],
            has_custom_textures: false,
        },
    }
}

fn make_profile(name: &str, sort_order: i32) -> ParsedProfile {
    ParsedProfile {
        name: name.to_string(),
        inherits_from: None,
        options: vec![],
        sort_order,
    }
}

fn expected_profile_id(version_id: &str, profile_name: &str) -> String {
    let input = format!("{version_id}/{profile_name}");
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(hash)
}

#[sqlx::test]
async fn test_list_profiles_empty(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.is_empty());
}

#[sqlx::test]
async fn test_list_profiles_returns_all(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_profile(&pool, "p1", "shv1", "Low", 0).await;
    seed_profile(&pool, "p2", "shv1", "Medium", 1).await;
    seed_profile(&pool, "p3", "shv1", "High", 2).await;

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.len() == 3);
}

#[sqlx::test]
async fn test_list_profiles_ordered_by_sort_order(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    // Insert in non-sequential order
    seed_profile(&pool, "p1", "shv1", "High", 2).await;
    seed_profile(&pool, "p2", "shv1", "Low", 0).await;
    seed_profile(&pool, "p3", "shv1", "Medium", 1).await;

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.len() == 3);
    check!(profiles[0].sort_order == 0);
    check!(profiles[1].sort_order == 1);
    check!(profiles[2].sort_order == 2);
    check!(profiles[0].name == "Low");
    check!(profiles[1].name == "Medium");
    check!(profiles[2].name == "High");
}

#[sqlx::test]
async fn test_list_profiles_scoped_to_version(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader_version(&pool, "shv2", "sh1", "2.0.0").await;

    seed_profile(&pool, "p1", "shv1", "Low", 0).await;
    seed_profile(&pool, "p2", "shv1", "High", 1).await;
    seed_profile(&pool, "p3", "shv2", "Ultra", 0).await;

    let v1_profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list v1 profiles");
    let v2_profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv2")
        .await
        .expect("list v2 profiles");

    assert!(v1_profiles.len() == 2);
    assert!(v2_profiles.len() == 1);
    check!(v2_profiles[0].name == "Ultra");
}

#[sqlx::test]
async fn test_deterministic_profile_id_is_sha256(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let data = make_shader_pack_data(vec![make_profile("Low", 0)], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.len() == 1);

    let expected_id = expected_profile_id("shv1", "Low");
    check!(profiles[0].id.0 == expected_id);
}

#[sqlx::test]
async fn test_deterministic_profile_id_stable_across_re_extraction(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let data = make_shader_pack_data(vec![make_profile("Low", 0), make_profile("High", 1)], None);

    // First extraction
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction 1");
    let first = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles 1");

    // Second extraction with same data
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction 2");
    let second = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles 2");

    assert!(first.len() == 2);
    assert!(second.len() == 2);
    check!(first[0].id == second[0].id);
    check!(first[1].id == second[1].id);
}

#[sqlx::test]
async fn test_persist_extraction_inserts_profiles(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let data = make_shader_pack_data(vec![make_profile("Low", 0), make_profile("High", 1)], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.len() == 2);
    check!(profiles[0].name == "Low");
    check!(profiles[0].sort_order == 0);
    check!(profiles[1].name == "High");
    check!(profiles[1].sort_order == 1);
}

#[sqlx::test]
async fn test_persist_extraction_replaces_existing(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // First extraction: Low + High
    let data1 = make_shader_pack_data(vec![make_profile("Low", 0), make_profile("High", 1)], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data1)
        .await
        .expect("persist extraction 1");

    // Second extraction: only Ultra
    let data2 = make_shader_pack_data(vec![make_profile("Ultra", 0)], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data2)
        .await
        .expect("persist extraction 2");

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.len() == 1);
    check!(profiles[0].name == "Ultra");
}

#[sqlx::test]
async fn test_persist_extraction_with_no_profiles_clears_existing(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // First: insert profiles
    let data1 = make_shader_pack_data(vec![make_profile("Low", 0), make_profile("High", 1)], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data1)
        .await
        .expect("persist extraction 1");

    // Second: no properties at all
    let data2 = ShaderPackData {
        properties: None,
        lang: None,
        scan: ZipScanResult {
            file_paths: vec!["shaders/gbuffers_terrain.vsh".into()],
            dimension_support: vec!["overworld".into()],
            has_custom_textures: false,
        },
    };
    ExtractionRepo::persist_extraction(&pool, "shv1", &data2)
        .await
        .expect("persist extraction 2");

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.is_empty());
}

#[sqlx::test]
async fn test_persist_extraction_upserts_metadata(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // First extraction with custom textures = false
    let data1 = ShaderPackData {
        properties: None,
        lang: None,
        scan: ZipScanResult {
            file_paths: vec!["shaders/gbuffers_terrain.vsh".into()],
            dimension_support: vec!["overworld".into()],
            has_custom_textures: false,
        },
    };
    ExtractionRepo::persist_extraction(&pool, "shv1", &data1)
        .await
        .expect("persist extraction 1");

    // Second extraction with custom textures = true and different paths
    let data2 = ShaderPackData {
        properties: None,
        lang: None,
        scan: ZipScanResult {
            file_paths: vec![
                "shaders/gbuffers_terrain.vsh".into(),
                "shaders/composite.fsh".into(),
            ],
            dimension_support: vec!["overworld".into(), "nether".into()],
            has_custom_textures: true,
        },
    };
    ExtractionRepo::persist_extraction(&pool, "shv1", &data2)
        .await
        .expect("persist extraction 2");

    let metadata = ExtractionRepo::get_metadata_by_version(&pool, "shv1")
        .await
        .expect("get metadata");
    let_assert!(Some(meta) = metadata);
    check!(meta.shader_version_id.0 == "shv1");
    check!(meta.has_custom_textures == Some(true));

    // Verify file_paths contains the second extraction's data
    let_assert!(Some(paths) = &meta.file_paths);
    let paths_arr = paths.as_array().expect("file_paths should be array");
    check!(paths_arr.len() == 2);

    // Verify only 1 metadata row exists
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM shader_version_metadata WHERE shader_version_id = 'shv1'",
    )
    .fetch_one(&pool)
    .await
    .expect("count metadata");
    check!(count.0 == 1);
}

#[sqlx::test]
async fn test_persist_extraction_sets_status_completed(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let data = make_shader_pack_data(vec![], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    let row: (String,) =
        sqlx::query_as("SELECT extraction_status FROM shader_versions WHERE id = 'shv1'")
            .fetch_one(&pool)
            .await
            .expect("query status");
    check!(row.0 == "completed");
}

#[sqlx::test]
async fn test_persist_extraction_clears_error(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Manually set an extraction error
    sqlx::query(
        "UPDATE shader_versions SET extraction_error = 'something broke' WHERE id = 'shv1'",
    )
    .execute(&pool)
    .await
    .expect("set error");

    let data = make_shader_pack_data(vec![], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    let row: (Option<String>,) =
        sqlx::query_as("SELECT extraction_error FROM shader_versions WHERE id = 'shv1'")
            .fetch_one(&pool)
            .await
            .expect("query error");
    check!(row.0.is_none());
}

#[sqlx::test]
async fn test_persist_extraction_with_profile_options(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let profile = ParsedProfile {
        name: "Custom".to_string(),
        inherits_from: None,
        options: vec![
            ProfileOption::Enable("SHADOWS".into()),
            ProfileOption::Disable("DOF".into()),
            ProfileOption::Set("shadowMapResolution".into(), "2048".into()),
        ],
        sort_order: 0,
    };

    let data = make_shader_pack_data(vec![profile], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.len() == 1);

    let opts = profiles[0]
        .options
        .as_object()
        .expect("options should be object");
    check!(opts.get("SHADOWS").expect("SHADOWS") == "true");
    check!(opts.get("DOF").expect("DOF") == "false");
    check!(
        opts.get("shadowMapResolution")
            .expect("shadowMapResolution")
            == "2048"
    );
}

#[sqlx::test]
async fn test_persist_extraction_with_lang_labels(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let mut lang = LangData::default();
    lang.profile_labels
        .insert("Low".into(), "Low Quality".into());
    lang.profile_labels
        .insert("High".into(), "High Quality".into());

    let data = make_shader_pack_data(
        vec![make_profile("Low", 0), make_profile("High", 1)],
        Some(lang),
    );
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    let profiles = ExtractionRepo::list_profiles_by_version(&pool, "shv1")
        .await
        .expect("list profiles");
    assert!(profiles.len() == 2);
    check!(profiles[0].label == Some("Low Quality".into()));
    check!(profiles[1].label == Some("High Quality".into()));
}

#[sqlx::test]
async fn test_get_metadata_none_when_not_extracted(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let metadata = ExtractionRepo::get_metadata_by_version(&pool, "shv1")
        .await
        .expect("get metadata");
    assert!(metadata.is_none());
}

#[sqlx::test]
async fn test_get_metadata_returns_data_after_extraction(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let data = make_shader_pack_data(vec![], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    let metadata = ExtractionRepo::get_metadata_by_version(&pool, "shv1")
        .await
        .expect("get metadata");
    let_assert!(Some(meta) = metadata);
    check!(meta.shader_version_id.0 == "shv1");
}

#[sqlx::test]
async fn test_reset_extraction_status_sets_pending(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Complete an extraction first
    let data = make_shader_pack_data(vec![], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    // Verify it's completed
    let row: (String,) =
        sqlx::query_as("SELECT extraction_status FROM shader_versions WHERE id = 'shv1'")
            .fetch_one(&pool)
            .await
            .expect("query status");
    check!(row.0 == "completed");

    // Reset
    let result = ExtractionRepo::reset_extraction_status(&pool, "shv1")
        .await
        .expect("reset status");
    assert!(result);

    // Verify pending
    let row: (String, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT extraction_status, extracted_at FROM shader_versions WHERE id = 'shv1'",
    )
    .fetch_one(&pool)
    .await
    .expect("query status after reset");
    check!(row.0 == "pending");
    check!(row.1.is_none());
}

#[sqlx::test]
async fn test_reset_extraction_status_returns_false_for_nonexistent(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    let result = ExtractionRepo::reset_extraction_status(&pool, "nonexistent-version")
        .await
        .expect("reset status");
    assert!(!result);
}

#[sqlx::test]
async fn test_adding_profiles_changes_work_items(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // Set up world + scene
    setup_basic_world_and_scene(&pool).await;

    // Create a custom shader (non-vanilla) with no profiles
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Work items before profiles: custom shader gets 1 item (NULL profile)
    let items_before = glint::repo::WorkRepo::get_work_items(&pool, 100, true, None, None)
        .await
        .expect("get work items before");
    let custom_before: Vec<_> = items_before
        .iter()
        .filter(|i| i.shader_version_id.0 == "shv1")
        .collect();
    check!(custom_before.len() == 1);
    check!(custom_before[0].profile_id.is_none());

    // Add 2 profiles via extraction
    let data = make_shader_pack_data(vec![make_profile("Low", 0), make_profile("High", 1)], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    // Work items after profiles: custom shader gets 2 items (one per profile)
    let items_after = glint::repo::WorkRepo::get_work_items(&pool, 100, true, None, None)
        .await
        .expect("get work items after");
    let custom_after: Vec<_> = items_after
        .iter()
        .filter(|i| i.shader_version_id.0 == "shv1")
        .collect();
    check!(custom_after.len() == 2);
    // Both should have profile IDs
    assert!(custom_after.iter().all(|i| i.profile_id.is_some()));
}

#[sqlx::test]
async fn test_adding_profiles_changes_health_targets(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // Set up world + scene
    setup_basic_world_and_scene(&pool).await;

    // Create a custom shader with no profiles
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Health targets before profiles
    let health_before = glint::repo::CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get health before");
    let custom_before: Vec<_> = health_before
        .targets
        .iter()
        .filter(|t| t.shader_version_id.0 == "shv1")
        .collect();
    check!(custom_before.len() == 1);
    check!(custom_before[0].profile_id.is_none());

    // Add 2 profiles via extraction
    let data = make_shader_pack_data(vec![make_profile("Low", 0), make_profile("High", 1)], None);
    ExtractionRepo::persist_extraction(&pool, "shv1", &data)
        .await
        .expect("persist extraction");

    // Health targets after profiles
    let health_after = glint::repo::CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get health after");
    let custom_after: Vec<_> = health_after
        .targets
        .iter()
        .filter(|t| t.shader_version_id.0 == "shv1")
        .collect();
    check!(custom_after.len() == 2);
    assert!(custom_after.iter().all(|t| t.profile_id.is_some()));
}
