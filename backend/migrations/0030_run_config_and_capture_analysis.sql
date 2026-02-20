-- Run-level capture configuration and per-capture analysis metadata.
--
-- resolution_width/height/image_format on capture_runs define the expected
-- output for all captures in a run, enabling validation at upload time.
-- analysis on captures stores per-image validation/processing results.

ALTER TABLE capture_runs
ADD COLUMN resolution_width  INTEGER NOT NULL DEFAULT 3840,
ADD COLUMN resolution_height INTEGER NOT NULL DEFAULT 2160,
ADD COLUMN image_format      TEXT    NOT NULL DEFAULT 'webp';

-- Backfill is handled by the DEFAULT clause above.
-- Drop the defaults so future INSERTs must supply values explicitly.
ALTER TABLE capture_runs
ALTER COLUMN resolution_width  DROP DEFAULT,
ALTER COLUMN resolution_height DROP DEFAULT,
ALTER COLUMN image_format      DROP DEFAULT;

ALTER TABLE captures
ADD COLUMN analysis JSONB;
