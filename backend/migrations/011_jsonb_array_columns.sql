-- Convert JSON-encoded TEXT columns to native JSONB arrays.
-- Existing rows contain JSON strings like '["1.8.9","1.9"]' which cast cleanly.
-- NULL values remain NULL.

ALTER TABLE shader_versions
    ALTER COLUMN game_versions TYPE JSONB USING game_versions::jsonb,
    ALTER COLUMN supported_profiles TYPE JSONB USING supported_profiles::jsonb;
