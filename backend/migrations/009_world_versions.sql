-- Create world_versions table
CREATE TABLE world_versions (
    id TEXT PRIMARY KEY,
    world_id TEXT NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
    file_url TEXT,
    file_hash TEXT,
    size_bytes BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_world_versions_world_created ON world_versions(world_id, created_at DESC);

-- Migrate existing world artifact data into world_versions
INSERT INTO world_versions (id, world_id, file_url, file_hash, size_bytes, created_at)
SELECT
    gen_random_uuid()::text,
    id,
    file_url,
    file_hash,
    size_bytes,
    created_at
FROM worlds
WHERE file_url IS NOT NULL;

-- Add world_version_id to captures
ALTER TABLE captures ADD COLUMN world_version_id TEXT REFERENCES world_versions(id);

-- Backfill captures with the initial world version (via scene -> world -> world_version)
UPDATE captures c
SET world_version_id = wv.id
FROM scenes s
JOIN LATERAL (
    SELECT id FROM world_versions wv2
    WHERE wv2.world_id = s.world_id
    ORDER BY wv2.created_at DESC
    LIMIT 1
) wv ON TRUE
WHERE c.scene_id = s.id;

-- Drop outdated column and its index from captures
DROP INDEX IF EXISTS idx_captures_outdated;
ALTER TABLE captures DROP COLUMN outdated;

-- Drop artifact columns from worlds
ALTER TABLE worlds DROP COLUMN file_url;
ALTER TABLE worlds DROP COLUMN file_hash;
ALTER TABLE worlds DROP COLUMN size_bytes;
