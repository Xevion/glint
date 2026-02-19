use assert2::{assert, check, let_assert};
use glint::db::apply_views;
use glint::repo::WorkRepo;

mod helpers;
use helpers::*;

#[sqlx::test]
async fn test_work_items_empty_when_no_scenes(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // 1 custom shader, but no worlds/scenes at all
    seed_shader(&pool, "sh1", "cool-shader", "Cool Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    assert!(items.is_empty());
}

#[sqlx::test]
async fn test_work_items_only_vanilla_when_no_custom_shaders(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // 1 scene, only the vanilla shader from migration 001
    setup_basic_scene(&pool).await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    assert!(items.len() == 1);
    check!(items[0].shader.name == "Vanilla");
}

#[sqlx::test]
async fn test_work_items_basic_matrix_expansion(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // 2 custom shaders × 2 scenes + vanilla × 2 scenes = 6 items
    setup_two_scenes(&pool).await;
    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader(&pool, "sh2", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    assert!(items.len() == 6);
}

#[sqlx::test]
async fn test_work_items_inactive_scenes_excluded(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_scene(&pool, "sc1", "active-scene", "Active Scene", true).await;
    seed_scene_version(&pool, "sv1", "sc1").await;
    seed_scene(&pool, "sc2", "inactive-scene", "Inactive Scene", false).await;
    seed_scene_version(&pool, "sv2", "sc2").await;
    seed_scene_preset(&pool, "sp1", "sc1", "Default", "default", 6000).await;
    seed_scene_preset(&pool, "sp2", "sc2", "Default", "default-b", 6000).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    // Only the active scene: shader-a × active + vanilla × active = 2
    assert!(items.len() == 2);
    for item in &items {
        check!(item.scene.slug == "active-scene");
    }
}

#[sqlx::test]
async fn test_work_items_version_with_profiles_expands_per_profile(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_profile(&pool, "p1", "shv1", "Low", 0).await;
    seed_profile(&pool, "p2", "shv1", "Medium", 1).await;
    seed_profile(&pool, "p3", "shv1", "High", 2).await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    // 3 profiles × 1 scene for shader-a + 1 vanilla = 4
    assert!(items.len() == 4);

    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.len() == 3);
    for item in &custom_items {
        check!(item.shader.profile_id.is_some());
        check!(item.shader.profile_name.is_some());
    }

    let vanilla_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "vanilla")
        .collect();
    assert!(vanilla_items.len() == 1);
    check!(vanilla_items[0].shader.profile_id.is_none());
    check!(vanilla_items[0].shader.profile_name.is_none());
}

#[sqlx::test]
async fn test_work_items_version_without_profiles_has_null_profile_id(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    let custom = items.iter().find(|i| i.shader.slug == "shader-a").unwrap();
    check!(custom.shader.profile_id.is_none());
    check!(custom.shader.profile_name.is_none());
}

#[sqlx::test]
async fn test_work_items_mixed_profile_and_no_profile_shaders(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    // Shader A: 2 profiles
    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_profile(&pool, "p1", "shv1", "Low", 0).await;
    seed_profile(&pool, "p2", "shv1", "High", 1).await;

    // Shader B: no profiles
    seed_shader(&pool, "sh2", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    // A×2 + B×1 + vanilla×1 = 4
    assert!(items.len() == 4);

    let a_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(a_items.len() == 2);
    for item in &a_items {
        check!(item.shader.profile_id.is_some());
        check!(item.shader.profile_name.is_some());
    }
    let a_names: Vec<_> = a_items
        .iter()
        .filter_map(|i| i.shader.profile_name.as_deref())
        .collect();
    check!(a_names.contains(&"Low"));
    check!(a_names.contains(&"High"));

    let b_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-b")
        .collect();
    assert!(b_items.len() == 1);
    check!(b_items[0].shader.profile_id.is_none());
    check!(b_items[0].shader.profile_name.is_none());

    let vanilla_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "vanilla")
        .collect();
    assert!(vanilla_items.len() == 1);
    check!(vanilla_items[0].shader.profile_id.is_none());
    check!(vanilla_items[0].shader.profile_name.is_none());
}

#[sqlx::test]
async fn test_work_items_excludes_targets_with_fresh_captures(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_two_scenes(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Fresh capture for scene 1: matching scene_version_id
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2025, 1, 1),
        Some("sv1"),
    )
    .await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    // Custom shader should only appear for scene 2 (scene 1 has fresh capture)
    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.len() == 1);
    check!(custom_items[0].scene.slug == "scene-b");
}

#[sqlx::test]
async fn test_work_items_includes_targets_with_stale_captures(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_scene(&pool, "sc1", "scene-a", "Scene A", true).await;
    seed_scene_version(&pool, "sv-old", "sc1").await;
    seed_scene_preset(&pool, "sp1", "sc1", "Default", "default", 6000).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Capture with old scene version → stale
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2025, 1, 1),
        Some("sv-old"),
    )
    .await;

    // Add a new scene version → old capture is now stale
    seed_scene_version(&pool, "sv-new", "sc1").await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.len() == 1);
    check!(custom_items[0].scene.slug == "scene-a");
}

#[sqlx::test]
async fn test_work_items_force_includes_fresh_captures(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Fresh capture
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2025, 1, 1),
        Some("sv1"),
    )
    .await;

    // force=false → excluded
    let items_no_force = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");
    let custom_no_force: Vec<_> = items_no_force
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_no_force.is_empty());

    // force=true → included
    let items_force = WorkRepo::get_work_items(&pool, 100, true, None, None, None)
        .await
        .expect("get_work_items");
    let custom_force: Vec<_> = items_force
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_force.len() == 1);
}

#[sqlx::test]
async fn test_work_items_null_profile_fresh_capture_excludes_correctly(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    // No profiles → NULL profile_id in matrix

    // Fresh capture with NULL profile_id
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None, // NULL profile_id
        Some("sp1"),
        "completed",
        ts(2025, 1, 1),
        Some("sv1"),
    )
    .await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    // Should be excluded via IS NOT DISTINCT FROM NULL matching
    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.is_empty());
}

#[sqlx::test]
async fn test_work_items_excludes_versions_at_failure_cap(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version_with_failures(&pool, "shv1", "sh1", "1.0.0", 3).await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.is_empty());
}

#[sqlx::test]
async fn test_work_items_includes_versions_below_failure_cap(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version_with_failures(&pool, "shv1", "sh1", "1.0.0", 2).await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.len() == 1);
}

#[sqlx::test]
async fn test_work_items_force_includes_failed_versions(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version_with_failures(&pool, "shv1", "sh1", "1.0.0", 5).await;

    // force=false → excluded
    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");
    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.is_empty());

    // force=true → included
    let items = WorkRepo::get_work_items(&pool, 100, true, None, None, None)
        .await
        .expect("get_work_items");
    let custom_items: Vec<_> = items
        .iter()
        .filter(|i| i.shader.slug == "shader-a")
        .collect();
    assert!(custom_items.len() == 1);
}

#[sqlx::test]
async fn test_work_items_shaders_filter_single_slug(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader(&pool, "sh2", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;

    let items =
        WorkRepo::get_work_items(&pool, 100, false, Some("shader-a".to_string()), None, None)
            .await
            .expect("get_work_items");

    // Only shader-a, vanilla excluded by filter
    assert!(items.len() == 1);
    check!(items[0].shader.slug == "shader-a");
}

#[sqlx::test]
async fn test_work_items_shaders_filter_comma_separated(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "slug-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader(&pool, "sh2", "slug-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;
    seed_shader(&pool, "sh3", "slug-c", "Shader C").await;
    seed_shader_version(&pool, "shv3", "sh3", "1.0.0").await;

    let items = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("slug-a,slug-b".to_string()),
        None,
        None,
    )
    .await
    .expect("get_work_items");

    // Only slug-a and slug-b
    assert!(items.len() == 2);
    let slugs: Vec<&str> = items.iter().map(|i| i.shader.slug.as_str()).collect();
    check!(slugs.contains(&"slug-a"));
    check!(slugs.contains(&"slug-b"));
    check!(!slugs.contains(&"slug-c"));
}

#[sqlx::test]
async fn test_work_items_scenes_filter_single_slug(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_two_scenes(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let items =
        WorkRepo::get_work_items(&pool, 100, false, None, Some("scene-a".to_string()), None)
            .await
            .expect("get_work_items");

    // shader-a × scene-a + vanilla × scene-a = 2
    assert!(items.len() == 2);
    for item in &items {
        check!(item.scene.slug == "scene-a");
    }
}

#[sqlx::test]
async fn test_work_items_both_filters_combined(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_two_scenes(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader(&pool, "sh2", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;

    let items = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("shader-a".to_string()),
        Some("scene-b".to_string()),
        None,
    )
    .await
    .expect("get_work_items");

    // Only shader-a × scene-b = 1
    assert!(items.len() == 1);
    check!(items[0].shader.slug == "shader-a");
    check!(items[0].scene.slug == "scene-b");
}

#[sqlx::test]
async fn test_work_items_uncaptured_versions_first(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    // Shader A: no captures
    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Shader B: has a stale capture (still shows up but lower priority)
    seed_shader(&pool, "sh2", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;
    seed_capture_full(
        &pool,
        "cap1",
        "shv2",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2025, 1, 1),
        Some("sv1"),
    )
    .await;
    // Add a new scene version so capture becomes stale
    seed_scene_version(&pool, "sv2", "sc1").await;

    let items = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("shader-a,shader-b".to_string()),
        None,
        None,
    )
    .await
    .expect("get_work_items");

    assert!(items.len() == 2);
    // Shader A (uncaptured) before Shader B (has captures)
    check!(items[0].shader.slug == "shader-a");
    check!(items[1].shader.slug == "shader-b");
}

#[sqlx::test]
async fn test_work_items_higher_downloads_first(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader_with_downloads(&pool, "sh1", "shader-a", "Shader A", 1000).await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader_with_downloads(&pool, "sh2", "shader-b", "Shader B", 500).await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;
    seed_shader_with_downloads(&pool, "sh3", "shader-c", "Shader C", 2000).await;
    seed_shader_version(&pool, "shv3", "sh3", "1.0.0").await;

    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    // Filter to only custom shaders for order check
    let custom: Vec<&str> = items
        .iter()
        .filter(|i| i.shader.slug != "vanilla")
        .map(|i| i.shader.slug.as_str())
        .collect();

    // C(2000) → A(1000) → B(500)
    assert!(custom.len() == 3);
    check!(custom[0] == "shader-c");
    check!(custom[1] == "shader-a");
    check!(custom[2] == "shader-b");

    // Vanilla (0 downloads) should be last overall
    let_assert!(Some(last) = items.last());
    check!(last.shader.slug == "vanilla");
}

#[sqlx::test]
async fn test_work_items_alphabetical_tiebreaker(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    seed_scene(&pool, "sc1", "forest", "Forest", true).await;
    seed_scene_version(&pool, "sv1", "sc1").await;
    seed_scene(&pool, "sc2", "sunset", "Sunset", true).await;
    seed_scene_version(&pool, "sv2", "sc2").await;
    seed_scene_preset(&pool, "sp1", "sc1", "Default", "default", 6000).await;
    seed_scene_preset(&pool, "sp2", "sc2", "Default", "default-b", 6000).await;

    // Same downloads so alphabetical tiebreaker applies
    seed_shader_with_downloads(&pool, "sh1", "beta-shader", "Beta", 100).await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader_with_downloads(&pool, "sh2", "alpha-shader", "Alpha", 100).await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;

    let items = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("alpha-shader,beta-shader".to_string()),
        None,
        None,
    )
    .await
    .expect("get_work_items");

    // Alpha/Forest, Alpha/Sunset, Beta/Forest, Beta/Sunset
    assert!(items.len() == 4);
    check!(items[0].shader.name == "Alpha");
    check!(items[0].scene.name == "Forest");
    check!(items[1].shader.name == "Alpha");
    check!(items[1].scene.name == "Sunset");
    check!(items[2].shader.name == "Beta");
    check!(items[2].scene.name == "Forest");
    check!(items[3].shader.name == "Beta");
    check!(items[3].scene.name == "Sunset");
}

#[sqlx::test]
async fn test_work_items_shader_limit_truncates(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_two_scenes(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader(&pool, "sh2", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;
    // Total: (2 custom + vanilla) × 2 scenes = 6

    // shader_limit=1 selects only the top shader → 1 shader × 2 scenes = 2
    let items = WorkRepo::get_work_items(&pool, 1, false, None, None, None)
        .await
        .expect("get_work_items");

    assert!(items.len() == 2);

    // shader_limit=2 selects top 2 shaders → 2 × 2 = 4
    let items = WorkRepo::get_work_items(&pool, 2, false, None, None, None)
        .await
        .expect("get_work_items");

    assert!(items.len() == 4);

    // shader_limit=100 selects all 3 shaders → 3 × 2 = 6
    let items = WorkRepo::get_work_items(&pool, 100, false, None, None, None)
        .await
        .expect("get_work_items");

    assert!(items.len() == 6);
}

#[sqlx::test]
async fn test_work_items_limit_zero_returns_empty(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let items = WorkRepo::get_work_items(&pool, 0, false, None, None, None)
        .await
        .expect("get_work_items");

    assert!(items.is_empty());
}

#[sqlx::test]
async fn test_adding_profiles_changes_work_items(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Initially no profiles → 1 item for custom shader
    let items =
        WorkRepo::get_work_items(&pool, 100, false, Some("shader-a".to_string()), None, None)
            .await
            .expect("get_work_items");
    assert!(items.len() == 1);
    check!(items[0].shader.profile_id.is_none());
    check!(items[0].shader.profile_name.is_none());

    // Add 2 profiles
    seed_profile(&pool, "p1", "shv1", "Low", 0).await;
    seed_profile(&pool, "p2", "shv1", "High", 1).await;

    let items =
        WorkRepo::get_work_items(&pool, 100, false, Some("shader-a".to_string()), None, None)
            .await
            .expect("get_work_items");
    assert!(items.len() == 2);
    for item in &items {
        check!(item.shader.profile_id.is_some());
        check!(item.shader.profile_name.is_some());
    }
}

#[sqlx::test]
async fn test_fresh_capture_for_one_profile_does_not_exclude_other(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_profile(&pool, "p-a", "shv1", "Profile A", 0).await;
    seed_profile(&pool, "p-b", "shv1", "Profile B", 1).await;

    // Fresh capture for profile A only
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        Some("p-a"),
        Some("sp1"),
        "completed",
        ts(2025, 1, 1),
        Some("sv1"),
    )
    .await;

    let items =
        WorkRepo::get_work_items(&pool, 100, false, Some("shader-a".to_string()), None, None)
            .await
            .expect("get_work_items");

    // Profile A excluded (fresh capture), Profile B still needed
    assert!(items.len() == 1);
    let_assert!(Some(profile_id) = &items[0].shader.profile_id);
    check!(profile_id.as_ref() == "p-b");
    check!(items[0].shader.profile_name.as_deref() == Some("Profile B"));
}

// -- Budget (target_items) tests --

#[sqlx::test]
async fn test_budget_single_shader_exceeding_target_included_whole(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // 1 shader × 3 scenes (each with a preset) = 3 items, target=1
    seed_scene(&pool, "sc1", "scene-a", "Scene A", true).await;
    seed_scene_version(&pool, "sv1", "sc1").await;
    seed_scene_preset(&pool, "sp1", "sc1", "Default", "default", 6000).await;
    seed_scene(&pool, "sc2", "scene-b", "Scene B", true).await;
    seed_scene_version(&pool, "sv2", "sc2").await;
    seed_scene_preset(&pool, "sp2", "sc2", "Default", "default-b", 6000).await;
    seed_scene(&pool, "sc3", "scene-c", "Scene C", true).await;
    seed_scene_version(&pool, "sv3", "sc3").await;
    seed_scene_preset(&pool, "sp3", "sc3", "Default", "default-c", 6000).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // target=1 but the shader has 3 items — must still return all 3
    let items = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("shader-a".to_string()),
        None,
        Some(1),
    )
    .await
    .expect("get_work_items");

    assert!(items.len() == 3);
    for item in &items {
        check!(item.shader.slug == "shader-a");
    }
}

#[sqlx::test]
async fn test_budget_stops_after_first_shader_crossing_target(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // 3 shaders × 2 scenes = 6 items total; each shader has 2 items.
    // target=2 → after shader 1 (2 items committed) we stop.
    setup_two_scenes(&pool).await;

    seed_shader_with_downloads(&pool, "sh1", "shader-a", "Shader A", 300).await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader_with_downloads(&pool, "sh2", "shader-b", "Shader B", 200).await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;
    seed_shader_with_downloads(&pool, "sh3", "shader-c", "Shader C", 100).await;
    seed_shader_version(&pool, "shv3", "sh3", "1.0.0").await;

    let items = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("shader-a,shader-b,shader-c".to_string()),
        None,
        Some(2),
    )
    .await
    .expect("get_work_items");

    // Only shader-a (highest priority, 2 items) should be included
    assert!(items.len() == 2);
    for item in &items {
        check!(item.shader.slug == "shader-a");
    }
}

#[sqlx::test]
async fn test_budget_accumulates_until_target(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    // 3 shaders × 1 scene = 3 items total (1 item each).
    // target=2 → after shader 1 (1 item, not yet at target), shader 2 (2 items, reached) → stop.
    setup_basic_scene(&pool).await;

    seed_shader_with_downloads(&pool, "sh1", "shader-a", "Shader A", 300).await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader_with_downloads(&pool, "sh2", "shader-b", "Shader B", 200).await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;
    seed_shader_with_downloads(&pool, "sh3", "shader-c", "Shader C", 100).await;
    seed_shader_version(&pool, "shv3", "sh3", "1.0.0").await;

    let items = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("shader-a,shader-b,shader-c".to_string()),
        None,
        Some(2),
    )
    .await
    .expect("get_work_items");

    // shader-a (1 item) + shader-b (1 item) = 2, hits target → stop before shader-c
    assert!(items.len() == 2);
    let slugs: Vec<&str> = items.iter().map(|i| i.shader.slug.as_str()).collect();
    check!(slugs.contains(&"shader-a"));
    check!(slugs.contains(&"shader-b"));
    check!(!slugs.contains(&"shader-c"));
}

#[sqlx::test]
async fn test_budget_large_target_returns_all(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");

    setup_two_scenes(&pool).await;

    seed_shader(&pool, "sh1", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader(&pool, "sh2", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;

    // target=9999 should not truncate anything; same as None
    let items_capped = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("shader-a,shader-b".to_string()),
        None,
        Some(9999),
    )
    .await
    .expect("get_work_items capped");

    let items_uncapped = WorkRepo::get_work_items(
        &pool,
        100,
        false,
        Some("shader-a,shader-b".to_string()),
        None,
        None,
    )
    .await
    .expect("get_work_items uncapped");

    assert!(items_capped.len() == items_uncapped.len());
}
