use glint::db::apply_views;
use glint::repo::capture_health::{StaleReason, TargetHealth};
use glint::repo::{CaptureHealthRepo, CaptureRunRepo};

use assert2::{check, let_assert};

mod helpers;
use helpers::*;

/// Reproduces the ColumnDecode { index: "9", source: UnexpectedNullError } bug.
///
/// When a shader version has NO profiles, `capture_target_matrix` emits a row
/// with `NULL profile_id`. The LEFT JOIN on `shader_version_profiles` then
/// yields NULL for `svp.name`, but SQLx's compile-time analysis sees that
/// `shader_version_profiles.name` is `NOT NULL` and generates a non-nullable
/// decoder — causing an UnexpectedNullError at runtime.
#[sqlx::test]
async fn capture_health_handles_null_profile_name(pool: sqlx::PgPool) {
    // Views are not part of migrations — apply them explicitly.
    apply_views(&pool).await.expect("failed to apply views");

    // Insert minimal seed data: one shader version (no profiles) + one active scene.
    // The `capture_target_matrix` view will produce a row with NULL profile_id for this
    // combination, which triggers the LEFT JOIN NULL on profile_name.
    // Seed data as individual statements (PostgreSQL prepared statements
    // don't support multiple commands in a single query).
    sqlx::query("INSERT INTO scenes (id, name, slug, active) VALUES ('sc1', 'Test Scene', 'test-scene', TRUE)")
        .execute(&pool).await.expect("insert scene");
    sqlx::query("INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks, weather, weather_intensity) VALUES ('sv1', 'sc1', 0, 64, 0, 0, 0, 6000, 'clear', 0)")
        .execute(&pool).await.expect("insert scene_version");
    sqlx::query("INSERT INTO scene_presets (id, scene_id, name, slug, time_of_day_ticks, weather, weather_intensity, sort_order, created_at, updated_at) VALUES ('sp1', 'sc1', 'Default', 'default', 6000, 'clear', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')")
        .execute(&pool).await.expect("insert scene_preset");
    sqlx::query(
        "INSERT INTO shaders (id, name, slug) VALUES ('sh1', 'Test Shader', 'test-shader')",
    )
    .execute(&pool)
    .await
    .expect("insert shader");
    sqlx::query(
        "INSERT INTO shader_versions (id, shader_id, version) VALUES ('shv1', 'sh1', '1.0.0')",
    )
    .execute(&pool)
    .await
    .expect("insert shader_version");

    // This call should succeed. Before the fix, it fails with:
    //   ColumnDecode { index: "9", source: UnexpectedNullError }
    let result = CaptureHealthRepo::get_capture_health(&pool).await;
    assert!(
        result.is_ok(),
        "capture health query failed: {:?}",
        result.as_ref().unwrap_err()
    );

    let health = result.unwrap();

    // The vanilla shader from migration 001 also appears in the matrix,
    // so we have 2 shader versions × 1 scene = 2 targets.
    assert_eq!(health.summary.total_targets, 2);
    assert_eq!(health.summary.missing, 2);

    // Find our test shader target (the one that exercises the NULL profile path).
    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target not found");
    assert_eq!(target.scene_name, "Test Scene");
    assert!(target.profile_id.is_none());
    assert!(target.profile_name.is_none());
}

/// Reproduces the column 5 UnexpectedNullError in list_items_with_context.
///
/// Same root cause as the capture health bug: `svp.name` from a LEFT JOIN on
/// `shader_version_profiles` is NULL when `profile_id` is NULL, but SQLx's
/// EXPLAIN VERBOSE analysis fails to propagate nullability for NOT NULL columns
/// on the right side of LEFT JOINs (known SQLx bug: #2127, #3202).
#[sqlx::test]
async fn run_items_with_context_handles_null_profile_name(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("failed to apply views");

    // Seed: scene, shader, shader_version (no profiles).
    sqlx::query("INSERT INTO scenes (id, name, slug, active) VALUES ('sc1', 'Test Scene', 'test-scene', TRUE)")
        .execute(&pool).await.expect("insert scene");
    sqlx::query(
        "INSERT INTO shaders (id, name, slug) VALUES ('sh1', 'Test Shader', 'test-shader')",
    )
    .execute(&pool)
    .await
    .expect("insert shader");
    sqlx::query(
        "INSERT INTO shader_versions (id, shader_id, version) VALUES ('shv1', 'sh1', '1.0.0')",
    )
    .execute(&pool)
    .await
    .expect("insert shader_version");

    // Create a capture run with one item that has NULL profile_id.
    sqlx::query("INSERT INTO capture_runs (id, status, total_items, resolution_width, resolution_height, image_format) VALUES ('run1', 'running', 1, 1920, 1080, 'webp')")
        .execute(&pool)
        .await
        .expect("insert capture_run");
    sqlx::query("INSERT INTO capture_run_items (id, run_id, shader_version_id, scene_id, profile_id, status) VALUES ('item1', 'run1', 'shv1', 'sc1', NULL, 'pending')")
        .execute(&pool).await.expect("insert capture_run_item");

    // This call should succeed. Before the fix, it fails with:
    //   ColumnDecode { index: "5", source: UnexpectedNullError }
    let result = CaptureRunRepo::list_items_with_context(&pool, "run1").await;
    assert!(
        result.is_ok(),
        "list_items_with_context failed: {:?}",
        result.as_ref().unwrap_err()
    );

    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].profile_id.is_none());
    assert!(items[0].profile_name.is_none());
    assert_eq!(items[0].shader_name, "Test Shader");
    assert_eq!(items[0].scene_name, "Test Scene");
}

/// Target with no captures → Missing.
#[sqlx::test]
async fn test_health_missing_when_no_captures(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    check!(target.status == TargetHealth::Missing);
    check!(target.last_capture_at.is_none());
    check!(target.stale_reason.is_none());
}

/// Fresh completed capture → Completed.
#[sqlx::test]
async fn test_health_completed_when_fresh_capture(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2026, 1, 15),
        Some("sv1"),
    )
    .await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    check!(target.status == TargetHealth::Completed);
    check!(target.last_capture_at.is_some());
    check!(target.stale_reason.is_none());
}

/// Capture with old scene version → Stale(SceneUpdated).
#[sqlx::test]
async fn test_health_stale_when_scene_updated(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Capture references the current scene version sv1
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2026, 1, 15),
        Some("sv1"),
    )
    .await;

    // Insert a newer scene version — makes sv1 outdated
    seed_scene_version(&pool, "sv2", "sc1").await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    check!(target.status == TargetHealth::Stale);
    let_assert!(Some(reason) = &target.stale_reason);
    check!(*reason == StaleReason::SceneUpdated);
}

/// capture_failure_count >= 3 → Failed.
#[sqlx::test]
async fn test_health_failed_when_failure_count_at_cap(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version_with_failures(&pool, "shv1", "sh1", "1.0.0", 3).await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    check!(target.status == TargetHealth::Failed);
    check!(target.failure_count == 3);
}

/// No captures + failure_count >= 3 → Failed (not Missing).
#[sqlx::test]
async fn test_health_failed_overrides_missing(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version_with_failures(&pool, "shv1", "sh1", "1.0.0", 4).await;
    // No captures inserted

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    check!(target.status == TargetHealth::Failed);
    check!(target.failure_count == 4);
    check!(target.last_capture_at.is_none());
}

/// Stale capture + failure_count >= 3 → Failed.
#[sqlx::test]
async fn test_health_failed_overrides_stale(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version_with_failures(&pool, "shv1", "sh1", "1.0.0", 3).await;

    // Capture with current versions
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2026, 1, 15),
        Some("sv1"),
    )
    .await;

    // Make it stale by adding a newer scene version
    seed_scene_version(&pool, "sv2", "sc1").await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    // Failed takes priority over Stale in the CASE expression
    check!(target.status == TargetHealth::Failed);
    check!(target.failure_count == 3);
}

/// Version without profiles → NULL profile_id/name.
#[sqlx::test]
async fn test_health_null_profile_id_and_name(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    check!(target.profile_id.is_none());
    check!(target.profile_name.is_none());
}

/// Version with profiles → populated profile fields.
#[sqlx::test]
async fn test_health_with_profiles_shows_info(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_profile(&pool, "prof-low", "shv1", "Low", 1).await;
    seed_profile(&pool, "prof-high", "shv1", "High", 2).await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let targets: Vec<_> = health
        .targets
        .iter()
        .filter(|t| t.shader_name == "Test Shader")
        .collect();

    // Shader with 2 profiles → 2 targets (one per profile)
    check!(targets.len() == 2);

    for t in &targets {
        check!(t.profile_id.is_some());
        check!(t.profile_name.is_some());
    }

    let profile_names: Vec<_> = targets
        .iter()
        .map(|t| t.profile_name.as_ref().unwrap().as_str())
        .collect();
    check!(profile_names.contains(&"Low"));
    check!(profile_names.contains(&"High"));
}

/// Shader A with 2 profiles, Shader B without, 1 scene → correct target counts.
#[sqlx::test]
async fn test_health_mixed_profiles_correct_count(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;

    // Shader A: 2 profiles
    seed_shader(&pool, "shA", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shvA", "shA", "1.0.0").await;
    seed_profile(&pool, "prof-low", "shvA", "Low", 1).await;
    seed_profile(&pool, "prof-high", "shvA", "High", 2).await;

    // Shader B: no profiles
    seed_shader(&pool, "shB", "shader-b", "Shader B").await;
    seed_shader_version(&pool, "shvB", "shB", "2.0.0").await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let a_count = health
        .targets
        .iter()
        .filter(|t| t.shader_name == "Shader A")
        .count();
    let b_count = health
        .targets
        .iter()
        .filter(|t| t.shader_name == "Shader B")
        .count();
    let vanilla_count = health
        .targets
        .iter()
        .filter(|t| t.shader_slug == VANILLA_SHADER_SLUG)
        .count();

    check!(a_count == 2); // 2 profiles × 1 scene
    check!(b_count == 1); // no profiles × 1 scene
    check!(vanilla_count == 1); // vanilla (no profiles) × 1 scene

    // Total = 2 + 1 + 1 = 4
    check!(health.summary.total_targets == 4);
}

/// All targets missing → counts match.
#[sqlx::test]
async fn test_health_summary_all_missing(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;

    seed_shader(&pool, "sh1", "shader-one", "Shader One").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;
    seed_shader(&pool, "sh2", "shader-two", "Shader Two").await;
    seed_shader_version(&pool, "shv2", "sh2", "1.0.0").await;
    // No captures

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    // 2 custom shaders + 1 vanilla = 3 targets, all missing
    check!(health.summary.total_targets == 3);
    check!(health.summary.missing == health.summary.total_targets);
    check!(health.summary.completed == 0);
    check!(health.summary.stale == 0);
    check!(health.summary.failed == 0);
}

/// Complex scenario with mixed statuses.
#[sqlx::test]
async fn test_health_summary_mixed_statuses(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;

    // Shader B: failure_count=3, no capture → Failed
    seed_shader(&pool, "shB", "shader-b", "Shader B").await;
    seed_shader_version_with_failures(&pool, "shvB", "shB", "2.0.0", 3).await;

    // Shader C: stale capture (old scene version) → Stale
    seed_shader(&pool, "shC", "shader-c", "Shader C").await;
    seed_shader_version(&pool, "shvC", "shC", "3.0.0").await;
    seed_capture_full(
        &pool,
        "capC",
        "shvC",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2026, 1, 10),
        Some("sv1"),
    )
    .await;
    // Add newer scene version to make Shader C stale
    seed_scene_version(&pool, "sv2", "sc1").await;

    // Shader A: fresh capture → Completed
    // Inserted AFTER sv2 so the capture references the latest scene version.
    seed_shader(&pool, "shA", "shader-a", "Shader A").await;
    seed_shader_version(&pool, "shvA", "shA", "1.0.0").await;
    seed_capture_full(
        &pool,
        "capA",
        "shvA",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2026, 1, 15),
        Some("sv2"),
    )
    .await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    // Count individual target statuses
    let a_target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Shader A")
        .expect("Shader A target");
    let b_target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Shader B")
        .expect("Shader B target");
    let c_target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Shader C")
        .expect("Shader C target");
    let vanilla_target = health
        .targets
        .iter()
        .find(|t| t.shader_slug == VANILLA_SHADER_SLUG)
        .expect("Vanilla target");

    // Shader A is Completed (fresh capture)
    check!(a_target.status == TargetHealth::Completed);
    // Shader B is Failed (failure_count >= 3)
    check!(b_target.status == TargetHealth::Failed);
    // Shader C is Stale (scene updated after capture)
    check!(c_target.status == TargetHealth::Stale);

    // Vanilla has no captures — Missing (not Stale).
    check!(vanilla_target.status == TargetHealth::Missing);

    // Verify summary counts match
    // A=Completed, B=Failed, C=Stale, Vanilla=Missing
    check!(health.summary.total_targets == 4);
    check!(health.summary.completed == 1);
    check!(health.summary.failed == 1);
    check!(health.summary.stale == 1);
    check!(health.summary.missing == 1);
}

/// Capture with status='failed' doesn't count as a valid capture.
#[sqlx::test]
async fn test_health_ignores_non_completed_captures(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Insert a failed capture — should be ignored by best_captures CTE
    seed_capture_full(
        &pool,
        "cap1",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "failed",
        ts(2026, 1, 15),
        Some("sv1"),
    )
    .await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    // Failed capture is ignored, so target is Missing
    check!(target.status == TargetHealth::Missing);
    check!(target.last_capture_at.is_none());
}

/// Multiple captures → most recent determines status.
#[sqlx::test]
async fn test_health_uses_most_recent_capture(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;
    seed_shader(&pool, "sh1", "test-shader", "Test Shader").await;
    seed_shader_version(&pool, "shv1", "sh1", "1.0.0").await;

    // Older capture: fresh at the time (references sv1)
    seed_capture_full(
        &pool,
        "cap-old",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2026, 1, 10),
        Some("sv1"),
    )
    .await;

    // Insert newer scene version
    seed_scene_version(&pool, "sv2", "sc1").await;

    // Newer capture: still references old sv1 (stale against new sv2)
    seed_capture_full(
        &pool,
        "cap-new",
        "shv1",
        "sc1",
        None,
        Some("sp1"),
        "completed",
        ts(2026, 1, 20),
        Some("sv1"),
    )
    .await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let target = health
        .targets
        .iter()
        .find(|t| t.shader_name == "Test Shader")
        .expect("test shader target");

    // The most recent capture (cap-new) is stale because sv2 is now the latest scene version
    check!(target.status == TargetHealth::Stale);
    let_assert!(Some(reason) = &target.stale_reason);
    check!(*reason == StaleReason::SceneUpdated);
}

/// Results ordered by shader name ASC.
#[sqlx::test]
async fn test_health_ordered_by_shader_name(pool: sqlx::PgPool) {
    apply_views(&pool).await.expect("views");
    setup_basic_scene(&pool).await;

    // Insert in non-alphabetical order
    seed_shader(&pool, "shC", "charlie", "Charlie").await;
    seed_shader_version(&pool, "shvC", "shC", "1.0.0").await;
    seed_shader(&pool, "shA", "alpha", "Alpha").await;
    seed_shader_version(&pool, "shvA", "shA", "1.0.0").await;
    seed_shader(&pool, "shB", "beta", "Beta").await;
    seed_shader_version(&pool, "shvB", "shB", "1.0.0").await;

    let health = CaptureHealthRepo::get_capture_health(&pool)
        .await
        .expect("get_capture_health");

    let names: Vec<&str> = health
        .targets
        .iter()
        .map(|t| t.shader_name.as_str())
        .collect();

    // Alpha before Beta before Charlie before Vanilla
    let alpha_pos = names.iter().position(|n| *n == "Alpha").expect("Alpha");
    let beta_pos = names.iter().position(|n| *n == "Beta").expect("Beta");
    let charlie_pos = names.iter().position(|n| *n == "Charlie").expect("Charlie");
    let vanilla_pos = names.iter().position(|n| *n == "Vanilla").expect("Vanilla");

    check!(alpha_pos < beta_pos);
    check!(beta_pos < charlie_pos);
    check!(charlie_pos < vanilla_pos);
}
