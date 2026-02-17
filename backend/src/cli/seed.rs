use sqlx::PgPool;
use tracing::info;

/// Seed the database with sample data
pub async fn run(pool: &PgPool) -> anyhow::Result<()> {
    info!("Seeding database...");

    // Clear existing data (cascades will handle related records)
    sqlx::query!("DELETE FROM capture_run_items")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM capture_runs")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM captures").execute(pool).await?;
    sqlx::query!("DELETE FROM scene_presets")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM scene_versions")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM scenes").execute(pool).await?;
    sqlx::query!("DELETE FROM shader_versions WHERE shader_id NOT IN (SELECT id FROM shaders WHERE slug = 'vanilla')")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM shaders WHERE slug != 'vanilla'")
        .execute(pool)
        .await?;

    // Seed sample scenes (identity + config tuples)
    let scenes = vec![
        (
            "scene-001",
            "Village Sunset",
            "village-sunset",
            "Village at sunset with warm lighting",
            "minecraft:overworld",
            // version config
            100.5_f64,
            64.0_f64,
            200.5_f64,
            -15.0_f64,
            90.0_f64,
            12000_i32, // Sunset
            "clear",
            0.0_f64,
            "minecraft:plains",
        ),
        (
            "scene-002",
            "Mountain Noon",
            "mountain-noon",
            "Mountain vista at midday",
            "minecraft:overworld",
            -50.0_f64,
            120.0_f64,
            -80.0_f64,
            -30.0_f64,
            180.0_f64,
            6000_i32, // Noon
            "clear",
            0.0_f64,
            "minecraft:mountains",
        ),
        (
            "scene-003",
            "Nether Portal",
            "nether-portal",
            "Active nether portal with particles",
            "minecraft:the_nether",
            0.0_f64,
            70.0_f64,
            0.0_f64,
            0.0_f64,
            0.0_f64,
            6000_i32,
            "clear",
            0.0_f64,
            "minecraft:nether_wastes",
        ),
    ];

    for (id, name, slug, desc, dimension, x, y, z, pitch, yaw, time, weather, intensity, biome) in
        &scenes
    {
        sqlx::query!(
            r#"
            INSERT INTO scenes (id, name, slug, description, dimension)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            *id,
            *name,
            *slug,
            *desc as &str,
            *dimension,
        )
        .execute(pool)
        .await?;

        let version_id = crate::id::generate_id();
        sqlx::query!(
            r#"
            INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks, weather, weather_intensity, biome)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            version_id,
            *id,
            *x,
            *y,
            *z,
            *pitch,
            *yaw,
            *time,
            *weather,
            *intensity,
            *biome,
        )
        .execute(pool)
        .await?;
    }

    info!(count = scenes.len(), "Seeded scenes");

    // Seed scene presets (time/weather variations per scene)
    let presets = vec![
        (
            "preset-001",
            "scene-001",
            "Default",
            "default",
            12000_i32,
            "clear",
            0.0_f64,
            0,
        ),
        (
            "preset-002",
            "scene-001",
            "Rainy",
            "rainy",
            12000_i32,
            "rain",
            0.8_f64,
            1,
        ),
        (
            "preset-003",
            "scene-002",
            "Default",
            "default",
            6000_i32,
            "clear",
            0.0_f64,
            0,
        ),
        (
            "preset-004",
            "scene-003",
            "Default",
            "default",
            6000_i32,
            "clear",
            0.0_f64,
            0,
        ),
    ];

    for (id, scene_id, name, slug, time, weather, intensity, sort_order) in &presets {
        sqlx::query!(
            r#"
            INSERT INTO scene_presets (id, scene_id, name, slug, time_of_day_ticks, weather, weather_intensity, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            *id,
            *scene_id,
            *name,
            *slug,
            *time,
            *weather,
            *intensity,
            *sort_order,
        )
        .execute(pool)
        .await?;
    }

    info!(count = presets.len(), "Seeded scene presets");

    // Seed sample shader
    sqlx::query!(
        r#"
        INSERT INTO shaders (id, name, slug, description, modrinth_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        "shader-001",
        "BSL Shaders",
        "bsl-shaders",
        "Popular shader pack with balanced performance and visuals",
        "Q1vvjJYV"
    )
    .execute(pool)
    .await?;

    info!("Seeded 1 shader");

    // Seed shader version
    sqlx::query!(
        r#"
        INSERT INTO shader_versions (id, shader_id, version, modrinth_version_id)
        VALUES ($1, $2, $3, $4)
        "#,
        "shader-version-001",
        "shader-001",
        "v8.2.09",
        "abcd1234"
    )
    .execute(pool)
    .await?;

    info!("Seeded 1 shader version");

    // Seed sample captures (pending status, with preset_id)
    let captures = vec![
        (
            "capture-001",
            "shader-version-001",
            "scene-001",
            "preset-001",
        ),
        (
            "capture-002",
            "shader-version-001",
            "scene-002",
            "preset-003",
        ),
        ("capture-003", "vanilla-1.21.4", "scene-001", "preset-001"),
        ("capture-004", "vanilla-1.21.4", "scene-002", "preset-003"),
    ];

    for (id, shader_version_id, scene_id, preset_id) in &captures {
        sqlx::query!(
            r#"
            INSERT INTO captures (id, shader_version_id, scene_id, preset_id, status)
            VALUES ($1, $2, $3, $4, 'completed')
            "#,
            *id,
            *shader_version_id,
            *scene_id,
            *preset_id,
        )
        .execute(pool)
        .await?;
    }

    info!(count = captures.len(), "Seeded captures");
    info!("Database seeded successfully!");

    Ok(())
}
