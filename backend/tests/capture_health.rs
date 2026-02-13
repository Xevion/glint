use glint::db::apply_views;
use glint::repo::{CaptureHealthRepo, CaptureRunRepo};

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
    sqlx::query("INSERT INTO worlds (id, slug, name, minecraft_version) VALUES ('w1', 'test-world', 'Test World', '1.21.4')")
        .execute(&pool).await.expect("insert world");
    sqlx::query("INSERT INTO world_versions (id, world_id) VALUES ('wv1', 'w1')")
        .execute(&pool)
        .await
        .expect("insert world_version");
    sqlx::query("INSERT INTO scenes (id, name, slug, world_id, active) VALUES ('sc1', 'Test Scene', 'test-scene', 'w1', TRUE)")
        .execute(&pool).await.expect("insert scene");
    sqlx::query("INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks) VALUES ('sv1', 'sc1', 0, 64, 0, 0, 0, 6000)")
        .execute(&pool).await.expect("insert scene_version");
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

    // Seed: world, scene, shader, shader_version (no profiles).
    sqlx::query("INSERT INTO worlds (id, slug, name, minecraft_version) VALUES ('w1', 'test-world', 'Test World', '1.21.4')")
        .execute(&pool).await.expect("insert world");
    sqlx::query("INSERT INTO scenes (id, name, slug, world_id, active) VALUES ('sc1', 'Test Scene', 'test-scene', 'w1', TRUE)")
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
    sqlx::query("INSERT INTO capture_runs (id, status, total_items) VALUES ('run1', 'running', 1)")
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
