-- Shaders table
CREATE TABLE shaders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    version TEXT NOT NULL,
    source_url TEXT NOT NULL,
    source_platform TEXT NOT NULL, -- 'modrinth' | 'curseforge'
    source_id TEXT NOT NULL,
    file_hash TEXT,

    -- Feature flags
    has_pbr BOOLEAN DEFAULT FALSE,
    has_volumetrics BOOLEAN DEFAULT FALSE,
    has_taa BOOLEAN DEFAULT FALSE,
    has_raytracing BOOLEAN DEFAULT FALSE,

    -- Metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Scenes table
CREATE TABLE scenes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    world_file TEXT NOT NULL,

    -- Camera settings
    x REAL NOT NULL,
    y REAL NOT NULL,
    z REAL NOT NULL,
    pitch REAL NOT NULL DEFAULT 0,
    yaw REAL NOT NULL DEFAULT 0,

    -- Environment
    time_of_day TEXT NOT NULL DEFAULT 'noon', -- 'dawn' | 'noon' | 'sunset' | 'night'
    weather TEXT NOT NULL DEFAULT 'clear', -- 'clear' | 'rain' | 'thunder'

    -- Tags
    tags TEXT, -- JSON array of strings

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Captures table (shader x scene combinations)
CREATE TABLE captures (
    id TEXT PRIMARY KEY,
    shader_id TEXT NOT NULL REFERENCES shaders(id) ON DELETE CASCADE,
    scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,

    -- Media URLs
    screenshot_url TEXT,
    video_url TEXT,
    thumbnail_url TEXT,

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
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending' | 'processing' | 'completed' | 'failed'
    error_message TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(shader_id, scene_id)
);

-- Jobs table (capture job queue)
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    shader_id TEXT NOT NULL REFERENCES shaders(id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 0,

    status TEXT NOT NULL DEFAULT 'pending', -- 'pending' | 'running' | 'completed' | 'failed'
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,

    started_at TEXT,
    completed_at TEXT,
    error_message TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX idx_shaders_slug ON shaders(slug);
CREATE INDEX idx_shaders_source ON shaders(source_platform, source_id);
CREATE INDEX idx_scenes_slug ON scenes(slug);
CREATE INDEX idx_captures_shader ON captures(shader_id);
CREATE INDEX idx_captures_scene ON captures(scene_id);
CREATE INDEX idx_captures_status ON captures(status);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_priority ON jobs(priority DESC, created_at ASC);
