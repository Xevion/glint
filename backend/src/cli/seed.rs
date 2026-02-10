use sqlx::PgPool;

/// Seed the database with sample data
pub async fn run(pool: &PgPool) -> anyhow::Result<()> {
    println!("Seeding database...");

    // Clear existing data (cascades will handle related records)
    sqlx::query!("DELETE FROM capture_run_items")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM capture_runs")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM captures").execute(pool).await?;
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
    sqlx::query!("DELETE FROM worlds").execute(pool).await?;

    // Seed sample world
    sqlx::query!(
        r#"
        INSERT INTO worlds (id, slug, name, description, minecraft_version)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        "world-001",
        "demo-world",
        "Demo World",
        "Sample world with various test scenes",
        "1.21.4"
    )
    .execute(pool)
    .await?;

    println!("  Seeded 1 world");

    // Seed sample scenes (identity + config tuples)
    let scenes = vec![
        (
            "scene-001",
            "Village Sunset",
            "village-sunset",
            "Village at sunset with warm lighting",
            "world-001",
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
            "world-001",
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
            "world-001",
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

    for (
        id,
        name,
        slug,
        desc,
        world_id,
        dimension,
        x,
        y,
        z,
        pitch,
        yaw,
        time,
        weather,
        intensity,
        biome,
    ) in &scenes
    {
        sqlx::query!(
            r#"
            INSERT INTO scenes (id, name, slug, description, world_id, dimension)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            *id,
            *name,
            *slug,
            *desc as &str,
            *world_id,
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

    println!("  Seeded {} scenes", scenes.len());

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

    println!("  Seeded 1 shader");

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

    println!("  Seeded 1 shader version");

    // Seed sample captures (pending status)
    let captures = vec![
        ("capture-001", "shader-version-001", "scene-001"),
        ("capture-002", "shader-version-001", "scene-002"),
        ("capture-003", "vanilla-1.21.4", "scene-001"),
        ("capture-004", "vanilla-1.21.4", "scene-002"),
    ];

    for (id, shader_version_id, scene_id) in &captures {
        sqlx::query!(
            r#"
            INSERT INTO captures (id, shader_version_id, scene_id, status)
            VALUES ($1, $2, $3, 'pending')
            "#,
            *id,
            *shader_version_id,
            *scene_id
        )
        .execute(pool)
        .await?;
    }

    println!("  Seeded {} captures", captures.len());
    println!("Database seeded successfully!");

    Ok(())
}
