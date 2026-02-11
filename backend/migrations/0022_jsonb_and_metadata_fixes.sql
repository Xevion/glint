-- Convert metadata_json from TEXT to JSONB for proper typed JSON handling
ALTER TABLE capture_runs
    ALTER COLUMN metadata_json TYPE JSONB USING metadata_json::jsonb;

-- Fix corrupted file_size_bytes: NULL out bogus 0-byte values caused by
-- missing Content-Length headers on HEAD responses
UPDATE captures SET file_size_bytes = NULL WHERE file_size_bytes = 0;
