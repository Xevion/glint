--------------------------------------------------------------------------------
-- WORLDS TABLE
-- Downloadable world files containing scenes
--------------------------------------------------------------------------------

CREATE TABLE worlds (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    minecraft_version TEXT NOT NULL,
    file_url TEXT,
    file_hash TEXT,
    size_bytes INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_worlds_slug ON worlds(slug);

--------------------------------------------------------------------------------
-- SHADERS TABLE
-- Shader packs (identity, not version-specific)
--------------------------------------------------------------------------------

CREATE TABLE shaders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    modrinth_id TEXT,
    curseforge_id TEXT,
    website_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_shaders_slug ON shaders(slug);

--------------------------------------------------------------------------------
-- SHADER_VERSIONS TABLE
-- Specific releases of shader packs
--------------------------------------------------------------------------------

CREATE TABLE shader_versions (
    id TEXT PRIMARY KEY,
    shader_id TEXT NOT NULL REFERENCES shaders(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    modrinth_version_id TEXT,
    download_url TEXT,
    file_hash TEXT,
    supported_profiles TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    UNIQUE(shader_id, version)
);

CREATE INDEX idx_shader_versions_shader ON shader_versions(shader_id);

--------------------------------------------------------------------------------
-- SCENES TABLE
-- Camera positions and environment settings within worlds
--------------------------------------------------------------------------------

CREATE TABLE scenes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    world_id TEXT NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,

    -- Camera settings
    x REAL NOT NULL,
    y REAL NOT NULL,
    z REAL NOT NULL,
    pitch REAL NOT NULL DEFAULT 0,
    yaw REAL NOT NULL DEFAULT 0,

    -- Environment
    dimension TEXT NOT NULL DEFAULT 'minecraft:overworld',
    time_of_day_ticks INTEGER NOT NULL DEFAULT 6000,
    weather TEXT NOT NULL DEFAULT 'clear',
    weather_intensity REAL NOT NULL DEFAULT 0.0,
    moon_phase INTEGER,
    biome TEXT,

    -- Configuration
    definition_json TEXT,
    tags TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_scenes_slug ON scenes(slug);
CREATE INDEX idx_scenes_world ON scenes(world_id);

--------------------------------------------------------------------------------
-- CAPTURES TABLE
-- Screenshots/videos for shader_version x scene x profile combinations
--------------------------------------------------------------------------------

CREATE TABLE captures (
    id TEXT PRIMARY KEY,
    shader_version_id TEXT NOT NULL REFERENCES shader_versions(id) ON DELETE CASCADE,
    scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
    profile TEXT,

    -- Media URLs
    screenshot_url TEXT,
    screenshot_path TEXT,
    video_url TEXT,
    thumbnail_url TEXT,

    -- Screenshot metadata
    resolution_width INTEGER,
    resolution_height INTEGER,
    captured_at TEXT,

    -- Performance metrics
    avg_fps REAL,
    min_fps REAL,
    max_fps REAL,
    frame_time_avg REAL,
    frame_time_p99 REAL,

    -- Capture metadata
    minecraft_version TEXT,
    iris_version TEXT,
    gpu_model TEXT,

    -- Status
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    UNIQUE(shader_version_id, scene_id, profile)
);

CREATE INDEX idx_captures_shader_version ON captures(shader_version_id);
CREATE INDEX idx_captures_scene ON captures(scene_id);
CREATE INDEX idx_captures_status ON captures(status);

--------------------------------------------------------------------------------
-- JOBS TABLE
-- Capture job queue for agent orchestration
--------------------------------------------------------------------------------

CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    shader_version_id TEXT NOT NULL REFERENCES shader_versions(id) ON DELETE CASCADE,
    scene_ids TEXT,
    profiles TEXT,
    priority INTEGER NOT NULL DEFAULT 0,

    status TEXT NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,

    agent_id TEXT,
    claimed_at TEXT,
    last_heartbeat TEXT,
    started_at TEXT,
    completed_at TEXT,
    error_message TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_priority ON jobs(priority DESC, created_at ASC);
CREATE INDEX idx_jobs_agent ON jobs(agent_id);
CREATE INDEX idx_jobs_heartbeat ON jobs(last_heartbeat);

--------------------------------------------------------------------------------
-- SEED: VANILLA SHADER
-- Well-known shader entry for vanilla Minecraft captures
--------------------------------------------------------------------------------

-- Well-known vanilla shader constant ID
INSERT INTO shaders (id, name, slug, description, website_url, created_at, updated_at)
VALUES (
    'vanilla',
    'Vanilla',
    'vanilla',
    'Default Minecraft rendering without shaders',
    'https://www.minecraft.net',
    datetime('now', 'utc'),
    datetime('now', 'utc')
);

INSERT INTO shader_versions (id, shader_id, version, created_at)
VALUES (
    'vanilla-1.21.4',
    'vanilla',
    '1.21.4',
    datetime('now', 'utc')
);
