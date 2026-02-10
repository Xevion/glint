-- Create scene_versions table (config extracted from scenes)
CREATE TABLE scene_versions (
    id TEXT PRIMARY KEY,
    scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    z DOUBLE PRECISION NOT NULL,
    pitch DOUBLE PRECISION NOT NULL,
    yaw DOUBLE PRECISION NOT NULL,
    time_of_day_ticks INTEGER NOT NULL,
    weather TEXT NOT NULL DEFAULT 'clear',
    weather_intensity DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    moon_phase INTEGER,
    biome TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_scene_versions_scene_created ON scene_versions(scene_id, created_at DESC);

-- Add parent_scene_id for derivative (variant) relationship
ALTER TABLE scenes ADD COLUMN parent_scene_id TEXT REFERENCES scenes(id);

-- Add scene_version_id to captures
ALTER TABLE captures ADD COLUMN scene_version_id TEXT REFERENCES scene_versions(id);

-- Migrate existing scene configs into initial scene_versions
INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks, weather, weather_intensity, moon_phase, biome, created_at)
SELECT
    gen_random_uuid()::text,
    id,
    x, y, z, pitch, yaw,
    time_of_day_ticks, weather, weather_intensity, moon_phase, biome,
    created_at
FROM scenes;

-- Backfill captures with initial scene versions
UPDATE captures c
SET scene_version_id = sv.id
FROM scenes s
JOIN LATERAL (
    SELECT id FROM scene_versions sv2
    WHERE sv2.scene_id = s.id
    ORDER BY sv2.created_at DESC
    LIMIT 1
) sv ON TRUE
WHERE c.scene_id = s.id;

-- Drop config columns and definition_json from scenes
ALTER TABLE scenes DROP COLUMN x;
ALTER TABLE scenes DROP COLUMN y;
ALTER TABLE scenes DROP COLUMN z;
ALTER TABLE scenes DROP COLUMN pitch;
ALTER TABLE scenes DROP COLUMN yaw;
ALTER TABLE scenes DROP COLUMN time_of_day_ticks;
ALTER TABLE scenes DROP COLUMN weather;
ALTER TABLE scenes DROP COLUMN weather_intensity;
ALTER TABLE scenes DROP COLUMN moon_phase;
ALTER TABLE scenes DROP COLUMN biome;
ALTER TABLE scenes DROP COLUMN definition_json;
