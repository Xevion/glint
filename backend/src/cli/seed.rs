use sqlx::SqlitePool;

/// Seed the database with sample data
pub async fn run(pool: &SqlitePool) -> anyhow::Result<()> {
    println!("Seeding database...");

    // Clear existing data (cascades will handle related records)
    sqlx::query("DELETE FROM jobs").execute(pool).await?;
    sqlx::query("DELETE FROM captures").execute(pool).await?;
    sqlx::query("DELETE FROM scenes").execute(pool).await?;
    sqlx::query("DELETE FROM shader_versions WHERE id != 'vanilla-1.21.4'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM shaders WHERE id != 'vanilla'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM worlds").execute(pool).await?;

    // Seed sample world
    sqlx::query(
        r#"
        INSERT INTO worlds (id, slug, name, description, minecraft_version)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind("world-001")
    .bind("demo-world")
    .bind("Demo World")
    .bind("Sample world with various test scenes")
    .bind("1.21.4")
    .execute(pool)
    .await?;

    println!("  Seeded 1 world");

    // Seed sample scenes
    let scenes = vec![
        (
            "scene-001",
            "Village Sunset",
            "village-sunset",
            "Village at sunset with warm lighting",
            "world-001",
            100.5,
            64.0,
            200.5,
            -15.0,
            90.0,
            "minecraft:overworld",
            12000, // Sunset
            "clear",
            0.0,
            "minecraft:plains",
        ),
        (
            "scene-002",
            "Mountain Noon",
            "mountain-noon",
            "Mountain vista at midday",
            "world-001",
            -50.0,
            120.0,
            -80.0,
            -30.0,
            180.0,
            "minecraft:overworld",
            6000, // Noon
            "clear",
            0.0,
            "minecraft:mountains",
        ),
        (
            "scene-003",
            "Nether Portal",
            "nether-portal",
            "Active nether portal with particles",
            "world-001",
            0.0,
            70.0,
            0.0,
            0.0,
            0.0,
            "minecraft:the_nether",
            6000, // Noon (not applicable in Nether)
            "clear",
            0.0,
            "minecraft:nether_wastes",
        ),
    ];

    for (
        id,
        name,
        slug,
        desc,
        world_id,
        x,
        y,
        z,
        pitch,
        yaw,
        dimension,
        time,
        weather,
        intensity,
        biome,
    ) in &scenes
    {
        sqlx::query(
            r#"
            INSERT INTO scenes (id, name, slug, description, world_id, x, y, z, pitch, yaw, dimension, time_of_day_ticks, weather, weather_intensity, biome)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(desc)
        .bind(world_id)
        .bind(x)
        .bind(y)
        .bind(z)
        .bind(pitch)
        .bind(yaw)
        .bind(dimension)
        .bind(time)
        .bind(weather)
        .bind(intensity)
        .bind(biome)
        .execute(pool)
        .await?;
    }

    println!("  Seeded {} scenes", scenes.len());

    // Seed sample shader
    sqlx::query(
        r#"
        INSERT INTO shaders (id, name, slug, description, modrinth_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind("shader-001")
    .bind("BSL Shaders")
    .bind("bsl-shaders")
    .bind("Popular shader pack with balanced performance and visuals")
    .bind("Q1vvjJYV")
    .execute(pool)
    .await?;

    println!("  Seeded 1 shader");

    // Seed shader version
    sqlx::query(
        r#"
        INSERT INTO shader_versions (id, shader_id, version, modrinth_version_id)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind("shader-version-001")
    .bind("shader-001")
    .bind("v8.2.09")
    .bind("abcd1234")
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
        sqlx::query(
            r#"
            INSERT INTO captures (id, shader_version_id, scene_id, status)
            VALUES ($1, $2, $3, 'pending')
            "#,
        )
        .bind(id)
        .bind(shader_version_id)
        .bind(scene_id)
        .execute(pool)
        .await?;
    }

    println!("  Seeded {} captures", captures.len());
    println!("Database seeded successfully!");

    Ok(())
}
