use assert2::{assert, check};
use glint::id::{PendingSceneUploadId, ScenePresetId};
use glint::repo::{PendingSceneUploadRepo, ScenePresetRepo, SceneVersionRepo};

mod helpers;

#[sqlx::test]
async fn test_create_pending_upload_for_new_scene(pool: sqlx::PgPool) {
    let id = PendingSceneUploadId::generate();
    let upload = PendingSceneUploadRepo::create_for_new_scene(
        &pool,
        &id,
        "Test Scene",
        "test-scene",
        "overworld",
        Some("A test scene"),
        "1.21.4",
        "abc123hash",
        1024 * 1024,
        "scene-packages/test-scene/upload-001.zip",
    )
    .await
    .expect("create pending upload");

    check!(upload.id == id.as_ref());
    check!(upload.scene_name.as_deref() == Some("Test Scene"));
    check!(upload.scene_slug.as_deref() == Some("test-scene"));
    check!(upload.scene_dimension.as_deref() == Some("overworld"));
    check!(upload.scene_description.as_deref() == Some("A test scene"));
    check!(upload.minecraft_version == "1.21.4");
    check!(upload.file_hash == "abc123hash");
    check!(upload.size_bytes == 1024 * 1024);
    check!(upload.scene_id.is_none()); // new scene, no existing scene_id
}

#[sqlx::test]
async fn test_create_pending_upload_for_existing_scene(pool: sqlx::PgPool) {
    // Create a scene first
    helpers::seed_scene(&pool, "sc1", "test-scene", "Test Scene", true).await;
    helpers::seed_scene_version(&pool, "sv1", "sc1").await;

    let id = PendingSceneUploadId::generate();
    let upload = PendingSceneUploadRepo::create_for_existing_scene(
        &pool,
        &id,
        "sc1",
        "1.21.4",
        "def456hash",
        2048 * 1024,
        "scene-packages/test-scene/upload-002.zip",
    )
    .await
    .expect("create pending upload for existing scene");

    check!(upload.scene_id.as_deref() == Some("sc1"));
    check!(upload.scene_name.is_none()); // existing scene, no name stored
    check!(upload.file_hash == "def456hash");
}

#[sqlx::test]
async fn test_find_pending_upload_by_id(pool: sqlx::PgPool) {
    let id = PendingSceneUploadId::generate();
    PendingSceneUploadRepo::create_for_new_scene(
        &pool,
        &id,
        "Scene",
        "scene",
        "overworld",
        None,
        "1.21.4",
        "hash123",
        1024,
        "key.zip",
    )
    .await
    .expect("create");

    let found = PendingSceneUploadRepo::find_by_id(&pool, id.as_ref())
        .await
        .expect("find");
    assert!(found.is_some());
    check!(found.unwrap().file_hash == "hash123");
}

#[sqlx::test]
async fn test_find_expired_upload_returns_none(pool: sqlx::PgPool) {
    let id = PendingSceneUploadId::generate();
    PendingSceneUploadRepo::create_for_new_scene(
        &pool,
        &id,
        "Scene",
        "scene",
        "overworld",
        None,
        "1.21.4",
        "hash123",
        1024,
        "key.zip",
    )
    .await
    .expect("create");

    // Expire the upload by backdating expires_at
    sqlx::query(
        "UPDATE pending_scene_uploads SET expires_at = now() - interval '1 hour' WHERE id = $1",
    )
    .bind(id.as_ref())
    .execute(&pool)
    .await
    .expect("backdate");

    let found = PendingSceneUploadRepo::find_by_id(&pool, id.as_ref())
        .await
        .expect("find");
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_cleanup_expired_deletes_old_records(pool: sqlx::PgPool) {
    // Create two uploads
    let id1 = PendingSceneUploadId::generate();
    let id2 = PendingSceneUploadId::generate();
    PendingSceneUploadRepo::create_for_new_scene(
        &pool,
        &id1,
        "Scene 1",
        "scene-1",
        "overworld",
        None,
        "1.21.4",
        "hash1",
        1024,
        "key1.zip",
    )
    .await
    .expect("create 1");
    PendingSceneUploadRepo::create_for_new_scene(
        &pool,
        &id2,
        "Scene 2",
        "scene-2",
        "overworld",
        None,
        "1.21.4",
        "hash2",
        1024,
        "key2.zip",
    )
    .await
    .expect("create 2");

    // Expire only the first one
    sqlx::query(
        "UPDATE pending_scene_uploads SET expires_at = now() - interval '1 hour' WHERE id = $1",
    )
    .bind(id1.as_ref())
    .execute(&pool)
    .await
    .expect("backdate");

    let deleted = PendingSceneUploadRepo::cleanup_expired(&pool)
        .await
        .expect("cleanup");
    check!(deleted == 1);

    // First should be gone, second still exists
    let found1 = PendingSceneUploadRepo::find_by_id(&pool, id1.as_ref())
        .await
        .expect("find 1");
    assert!(found1.is_none());
    let found2 = PendingSceneUploadRepo::find_by_id(&pool, id2.as_ref())
        .await
        .expect("find 2");
    assert!(found2.is_some());
}

#[sqlx::test]
async fn test_delete_pending_upload(pool: sqlx::PgPool) {
    let id = PendingSceneUploadId::generate();
    PendingSceneUploadRepo::create_for_new_scene(
        &pool,
        &id,
        "Scene",
        "scene",
        "overworld",
        None,
        "1.21.4",
        "hash123",
        1024,
        "key.zip",
    )
    .await
    .expect("create");

    let deleted = PendingSceneUploadRepo::delete(&pool, id.as_ref())
        .await
        .expect("delete");
    assert!(deleted);

    let found = PendingSceneUploadRepo::find_by_id(&pool, id.as_ref())
        .await
        .expect("find");
    assert!(found.is_none());
}

#[sqlx::test]
async fn test_scene_version_create_with_package(pool: sqlx::PgPool) {
    helpers::seed_scene(&pool, "sc1", "test-scene", "Test Scene", true).await;

    let version = SceneVersionRepo::create_with_package(
        &pool,
        "sv-pkg-1",
        "sc1",
        100.0,
        64.0,
        200.0, // x, y, z
        -10.0,
        45.0,           // pitch, yaw
        6000,           // time_of_day_ticks
        "clear",        // weather
        0.0,            // weather_intensity
        None,           // moon_phase
        Some("plains"), // biome
        "1.21.4",       // minecraft_version
        "https://r2.example.com/scene-packages/test-scene/pkg.zip",
        "hash789",
        5 * 1024 * 1024,
        90, // fov
        12, // render_distance
    )
    .await
    .expect("create version with package");

    check!(version.id.as_ref() == "sv-pkg-1");
    check!(version.scene_id.as_ref() == "sc1");
    check!(
        version.package_url.as_deref()
            == Some("https://r2.example.com/scene-packages/test-scene/pkg.zip")
    );
    check!(version.package_hash.as_deref() == Some("hash789"));
    check!(version.fov == 90);
    check!(version.render_distance == 12);
    check!(version.minecraft_version.as_deref() == Some("1.21.4"));
}

#[sqlx::test]
async fn test_scene_preset_created_for_new_scene(pool: sqlx::PgPool) {
    // Simulate what complete_scene_upload does for a new scene:
    // 1. Create scene
    // 2. Create version with package
    // 3. Create default preset
    helpers::seed_scene(&pool, "sc1", "test-scene", "Test Scene", true).await;

    let _version = SceneVersionRepo::create_with_package(
        &pool,
        "sv1",
        "sc1",
        0.0,
        64.0,
        0.0,
        0.0,
        0.0,
        6000,
        "clear",
        0.0,
        None,
        None,
        "1.21.4",
        "https://r2.example.com/pkg.zip",
        "hash",
        1024,
        70,
        16,
    )
    .await
    .expect("create version");

    let preset_id = ScenePresetId::generate();
    let preset = ScenePresetRepo::create(
        &pool,
        preset_id.as_ref(),
        "sc1",
        "Default",
        "default",
        6000,    // time_of_day_ticks from environment
        "clear", // weather
        0.0,     // weather_intensity
        None,    // moon_phase
        0,       // sort_order
    )
    .await
    .expect("create preset");

    check!(preset.name == "Default");
    check!(preset.slug == "default");
    check!(preset.time_of_day_ticks == 6000);
    check!(preset.weather == "clear");
    check!(preset.scene_id.as_ref() == "sc1");

    // Verify the preset is listed for the scene
    let presets = ScenePresetRepo::list_by_scene(&pool, "sc1")
        .await
        .expect("list presets");
    assert!(presets.len() == 1);
    check!(presets[0].id == preset.id);
}
