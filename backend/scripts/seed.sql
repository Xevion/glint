-- Sample seed data for development and testing
-- Run with: just db-seed

--------------------------------------------------------------------------------
-- SAMPLE WORLD
--------------------------------------------------------------------------------

INSERT OR IGNORE INTO worlds (id, slug, name, description, minecraft_version)
VALUES (
    'world-001',
    'demo-world',
    'Demo World',
    'Sample world with various test scenes',
    '1.21.4'
);

--------------------------------------------------------------------------------
-- SAMPLE SCENES
--------------------------------------------------------------------------------

INSERT OR IGNORE INTO scenes (id, name, slug, description, world_id, x, y, z, pitch, yaw, dimension, time_of_day_ticks, weather, weather_intensity, biome)
VALUES 
    (
        'scene-001',
        'Village Sunset',
        'village-sunset',
        'Village at sunset with warm lighting',
        'world-001',
        100.5, 64.0, 200.5,
        -15.0, 90.0,
        'minecraft:overworld',
        12000, -- Sunset
        'clear',
        0.0,
        'minecraft:plains'
    ),
    (
        'scene-002',
        'Mountain Noon',
        'mountain-noon',
        'Mountain vista at midday',
        'world-001',
        -50.0, 120.0, -80.0,
        -30.0, 180.0,
        'minecraft:overworld',
        6000, -- Noon
        'clear',
        0.0,
        'minecraft:mountains'
    ),
    (
        'scene-003',
        'Nether Portal',
        'nether-portal',
        'Active nether portal with particles',
        'world-001',
        0.0, 70.0, 0.0,
        0.0, 0.0,
        'minecraft:the_nether',
        6000, -- Noon (not applicable in Nether)
        'clear',
        0.0,
        'minecraft:nether_wastes'
    );

--------------------------------------------------------------------------------
-- SAMPLE SHADER
--------------------------------------------------------------------------------

INSERT OR IGNORE INTO shaders (id, name, slug, description, modrinth_id)
VALUES (
    'shader-001',
    'BSL Shaders',
    'bsl-shaders',
    'Popular shader pack with balanced performance and visuals',
    'Q1vvjJYV'
);

INSERT OR IGNORE INTO shader_versions (id, shader_id, version, modrinth_version_id)
VALUES (
    'shader-version-001',
    'shader-001',
    'v8.2.09',
    'abcd1234'
);

--------------------------------------------------------------------------------
-- SAMPLE CAPTURES (pending status)
--------------------------------------------------------------------------------

INSERT OR IGNORE INTO captures (id, shader_version_id, scene_id, status, created_at, updated_at)
VALUES 
    ('capture-001', 'shader-version-001', 'scene-001', 'pending', datetime('now', 'utc'), datetime('now', 'utc')),
    ('capture-002', 'shader-version-001', 'scene-002', 'pending', datetime('now', 'utc'), datetime('now', 'utc')),
    ('capture-003', 'vanilla-1.21.4', 'scene-001', 'pending', datetime('now', 'utc'), datetime('now', 'utc')),
    ('capture-004', 'vanilla-1.21.4', 'scene-002', 'pending', datetime('now', 'utc'), datetime('now', 'utc'));
