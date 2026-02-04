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
    size_bytes BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
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
    slug TEXT NOT NULL,
    description TEXT,
    world_id TEXT NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,

    -- Camera settings
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    z DOUBLE PRECISION NOT NULL,
    pitch DOUBLE PRECISION NOT NULL DEFAULT 0,
    yaw DOUBLE PRECISION NOT NULL DEFAULT 0,

    -- Environment
    dimension TEXT NOT NULL DEFAULT 'minecraft:overworld',
    time_of_day_ticks INTEGER NOT NULL DEFAULT 6000,
    weather TEXT NOT NULL DEFAULT 'clear',
    weather_intensity DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    moon_phase INTEGER,
    biome TEXT,

    -- Configuration
    definition_json TEXT,
    active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- World-scoped slug uniqueness for active scenes only
CREATE UNIQUE INDEX idx_scenes_world_slug ON scenes(world_id, slug) WHERE active = TRUE;
CREATE INDEX idx_scenes_world ON scenes(world_id);
CREATE INDEX idx_scenes_active ON scenes(active);

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
    captured_at TIMESTAMPTZ,

    -- Performance metrics
    avg_fps DOUBLE PRECISION,
    min_fps DOUBLE PRECISION,
    max_fps DOUBLE PRECISION,
    frame_time_avg DOUBLE PRECISION,
    frame_time_p99 DOUBLE PRECISION,

    -- Capture metadata
    minecraft_version TEXT,
    iris_version TEXT,
    gpu_model TEXT,

    -- Status
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    outdated BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(shader_version_id, scene_id, profile)
);

CREATE INDEX idx_captures_shader_version ON captures(shader_version_id);
CREATE INDEX idx_captures_scene ON captures(scene_id);
CREATE INDEX idx_captures_status ON captures(status);
CREATE INDEX idx_captures_outdated ON captures(outdated);

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
    claimed_at TIMESTAMPTZ,
    last_heartbeat TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_priority ON jobs(priority DESC, created_at ASC);
CREATE INDEX idx_jobs_agent ON jobs(agent_id);
CREATE INDEX idx_jobs_heartbeat ON jobs(last_heartbeat);

--------------------------------------------------------------------------------
-- PENDING_UPLOADS TABLE
-- Tracks world uploads in progress (presigned URL workflow)
--------------------------------------------------------------------------------

CREATE TABLE pending_uploads (
    upload_id TEXT PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    minecraft_version TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    upload_key TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pending_uploads_slug ON pending_uploads(slug);
CREATE INDEX idx_pending_uploads_expires_at ON pending_uploads(expires_at);

--------------------------------------------------------------------------------
-- SEED: VANILLA SHADER
-- Well-known shader entry for vanilla Minecraft captures
--------------------------------------------------------------------------------

INSERT INTO shaders (id, name, slug, description, website_url, created_at, updated_at)
VALUES (
    'vanilla',
    'Vanilla',
    'vanilla',
    'Default Minecraft rendering without shaders',
    'https://www.minecraft.net',
    now(),
    now()
);

INSERT INTO shader_versions (id, shader_id, version, created_at)
VALUES (
    'vanilla-1.21.4',
    'vanilla',
    '1.21.4',
    now()
);
