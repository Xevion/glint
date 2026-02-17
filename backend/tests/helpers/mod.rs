//! Shared test seed-data helpers for database integration tests.
//!
//! Every helper inserts minimal valid rows into the test database.
//! All functions take `&PgPool` and use raw SQL — no repo layer dependency.

// Each test binary uses a subset of helpers — suppress unused warnings.
#![allow(dead_code)]

use chrono::{DateTime, TimeZone, Utc};
use sqlx::PgPool;

// -- Constants for the vanilla shader seeded by migration 001 --

pub const VANILLA_SHADER_ID: &str = "vanilla";
pub const VANILLA_SHADER_SLUG: &str = "vanilla";
pub const VANILLA_VERSION_ID: &str = "vanilla-1.21.4";

// -- Scene helpers --

pub async fn seed_scene(pool: &PgPool, id: &str, slug: &str, name: &str, active: bool) {
    sqlx::query("INSERT INTO scenes (id, name, slug, active) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(active)
        .execute(pool)
        .await
        .expect("seed_scene");
}

pub async fn seed_scene_version(pool: &PgPool, id: &str, scene_id: &str) {
    sqlx::query(
        "INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks, weather, weather_intensity) \
         VALUES ($1, $2, 0, 64, 0, 0, 0, 6000, 'clear', 0)",
    )
    .bind(id)
    .bind(scene_id)
    .execute(pool)
    .await
    .expect("seed_scene_version");
}

// -- Shader helpers --

pub async fn seed_shader(pool: &PgPool, id: &str, slug: &str, name: &str) {
    sqlx::query("INSERT INTO shaders (id, name, slug) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(name)
        .bind(slug)
        .execute(pool)
        .await
        .expect("seed_shader");
}

pub async fn seed_shader_with_downloads(
    pool: &PgPool,
    id: &str,
    slug: &str,
    name: &str,
    downloads: i64,
) {
    sqlx::query("INSERT INTO shaders (id, name, slug, upstream_downloads) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(downloads)
        .execute(pool)
        .await
        .expect("seed_shader_with_downloads");
}

pub async fn seed_shader_version(pool: &PgPool, id: &str, shader_id: &str, version: &str) {
    sqlx::query("INSERT INTO shader_versions (id, shader_id, version) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(shader_id)
        .bind(version)
        .execute(pool)
        .await
        .expect("seed_shader_version");
}

pub async fn seed_shader_version_with_failures(
    pool: &PgPool,
    id: &str,
    shader_id: &str,
    version: &str,
    failure_count: i32,
) {
    sqlx::query(
        "INSERT INTO shader_versions (id, shader_id, version, capture_failure_count) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(shader_id)
    .bind(version)
    .bind(failure_count)
    .execute(pool)
    .await
    .expect("seed_shader_version_with_failures");
}

pub async fn seed_shader_version_with_published_at(
    pool: &PgPool,
    id: &str,
    shader_id: &str,
    version: &str,
    published_at: DateTime<Utc>,
) {
    sqlx::query(
        "INSERT INTO shader_versions (id, shader_id, version, upstream_published_at) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(shader_id)
    .bind(version)
    .bind(published_at)
    .execute(pool)
    .await
    .expect("seed_shader_version_with_published_at");
}

// -- Profile helpers --

pub async fn seed_profile(pool: &PgPool, id: &str, version_id: &str, name: &str, sort_order: i32) {
    sqlx::query(
        "INSERT INTO shader_version_profiles (id, shader_version_id, name, options, sort_order) \
         VALUES ($1, $2, $3, '{}'::jsonb, $4)",
    )
    .bind(id)
    .bind(version_id)
    .bind(name)
    .bind(sort_order)
    .execute(pool)
    .await
    .expect("seed_profile");
}

// -- Capture helpers --

/// Insert a minimal capture. `status` should be 'completed', 'uploading', 'failed', etc.
pub async fn seed_capture(
    pool: &PgPool,
    id: &str,
    shader_version_id: &str,
    scene_id: &str,
    profile_id: Option<&str>,
    preset_id: Option<&str>,
    status: &str,
) {
    sqlx::query(
        "INSERT INTO captures (id, shader_version_id, scene_id, profile_id, preset_id, status, captured_at) \
         VALUES ($1, $2, $3, $4, $5, $6, now())",
    )
    .bind(id)
    .bind(shader_version_id)
    .bind(scene_id)
    .bind(profile_id)
    .bind(preset_id)
    .bind(status)
    .execute(pool)
    .await
    .expect("seed_capture");
}

/// Insert a capture with full freshness-relevant fields.
pub async fn seed_capture_full(
    pool: &PgPool,
    id: &str,
    shader_version_id: &str,
    scene_id: &str,
    profile_id: Option<&str>,
    preset_id: Option<&str>,
    status: &str,
    captured_at: DateTime<Utc>,
    scene_version_id: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO captures (id, shader_version_id, scene_id, profile_id, preset_id, status, captured_at, scene_version_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(shader_version_id)
    .bind(scene_id)
    .bind(profile_id)
    .bind(preset_id)
    .bind(status)
    .bind(captured_at)
    .bind(scene_version_id)
    .execute(pool)
    .await
    .expect("seed_capture_full");
}

// -- Scene preset helpers --

pub async fn seed_scene_preset(
    pool: &PgPool,
    id: &str,
    scene_id: &str,
    name: &str,
    slug: &str,
    time_of_day_ticks: i32,
) {
    // Set created_at/updated_at to a fixed past timestamp so test captures
    // with ts(2025, ...) or ts(2026, ...) aren't marked stale due to
    // captured_at < preset.updated_at.
    sqlx::query(
        "INSERT INTO scene_presets (id, scene_id, name, slug, time_of_day_ticks, weather, weather_intensity, sort_order, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, 'clear', 0, 0, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
    )
    .bind(id)
    .bind(scene_id)
    .bind(name)
    .bind(slug)
    .bind(time_of_day_ticks)
    .execute(pool)
    .await
    .expect("seed_scene_preset");
}

// -- Composite helpers --

/// Create a minimal scene + scene_version + scene_preset.
/// Returns (scene_id, scene_version_id, preset_id).
pub async fn setup_basic_scene(pool: &PgPool) -> (&'static str, &'static str, &'static str) {
    seed_scene(pool, "sc1", "test-scene", "Test Scene", true).await;
    seed_scene_version(pool, "sv1", "sc1").await;
    seed_scene_preset(pool, "sp1", "sc1", "Default", "default", 6000).await;
    ("sc1", "sv1", "sp1")
}

/// Create two active scenes with scene versions and presets.
pub async fn setup_two_scenes(pool: &PgPool) {
    seed_scene(pool, "sc1", "scene-a", "Scene A", true).await;
    seed_scene_version(pool, "sv1", "sc1").await;
    seed_scene_preset(pool, "sp1", "sc1", "Default", "default", 6000).await;
    seed_scene(pool, "sc2", "scene-b", "Scene B", true).await;
    seed_scene_version(pool, "sv2", "sc2").await;
    seed_scene_preset(pool, "sp2", "sc2", "Default", "default-b", 6000).await;
}

/// Utility: a fixed timestamp for deterministic tests.
pub fn ts(year: i32, month: u32, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
}
