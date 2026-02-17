-- Scene packages: self-contained scene presets replace world-file-based scenes.
--
-- This migration:
-- 1. Creates scene_presets (time-of-day, weather, moon phase per scene)
-- 2. Adds package fields to scene_versions (url, hash, size, fov, render_distance)
-- 3. Adds preset_id FK to captures and capture_run_items
-- 4. Deletes all capture data (clean slate — no backfill path)
-- 5. Drops world-related tables, columns, constraints, and indexes
-- 6. Drops pending_uploads table

-- 0. Drop views that reference columns/tables we're about to remove ----------
-- views.sql re-applies all views on app startup, so this is safe.

DROP VIEW IF EXISTS capture_details CASCADE;
DROP VIEW IF EXISTS capture_contexts CASCADE;
DROP VIEW IF EXISTS captures_with_freshness CASCADE;
DROP VIEW IF EXISTS capture_target_matrix CASCADE;
DROP VIEW IF EXISTS latest_world_versions CASCADE;

-- 1. Create scene_presets table ----------------------------------------------

CREATE TABLE scene_presets (
    id              TEXT PRIMARY KEY,
    scene_id        TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    slug            TEXT NOT NULL,
    time_of_day_ticks INTEGER NOT NULL,
    weather         TEXT NOT NULL DEFAULT 'clear',
    weather_intensity DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    moon_phase      INTEGER,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(scene_id, slug)
);

CREATE INDEX idx_scene_presets_scene ON scene_presets(scene_id);

-- 2. Add package fields to scene_versions ------------------------------------

ALTER TABLE scene_versions
    ADD COLUMN package_url        TEXT,
    ADD COLUMN package_hash       TEXT,
    ADD COLUMN package_size_bytes BIGINT,
    ADD COLUMN fov                INTEGER NOT NULL DEFAULT 70,
    ADD COLUMN render_distance    INTEGER NOT NULL DEFAULT 16;

-- 3. Add preset_id to captures and capture_run_items -------------------------

ALTER TABLE captures
    ADD COLUMN preset_id TEXT REFERENCES scene_presets(id);

ALTER TABLE capture_run_items
    ADD COLUMN preset_id TEXT REFERENCES scene_presets(id);

CREATE INDEX idx_captures_preset_id ON captures(preset_id) WHERE preset_id IS NOT NULL;

-- 4. Delete all capture data (clean slate) -----------------------------------
-- Order matters: run items reference captures, captures reference runs via items.

DELETE FROM capture_run_items;
DELETE FROM captures;
DELETE FROM capture_runs;

-- 5. Drop world-related columns from scenes ----------------------------------

-- Drop indexes that reference world_id before dropping the column.
DROP INDEX IF EXISTS idx_scenes_world_slug;
DROP INDEX IF EXISTS idx_scenes_world;

-- Drop FK constraint and CHECK constraint on scenes.
ALTER TABLE scenes DROP CONSTRAINT IF EXISTS scenes_world_id_fkey;
ALTER TABLE scenes DROP COLUMN world_id;

-- Drop parent_scene_id (no longer used — presets replace variants).
ALTER TABLE scenes DROP CONSTRAINT IF EXISTS scenes_parent_scene_id_fkey;
ALTER TABLE scenes DROP COLUMN parent_scene_id;

-- Replace the old world-scoped slug uniqueness with a global one.
CREATE UNIQUE INDEX idx_scenes_slug_active ON scenes(slug) WHERE active = TRUE;

-- 6. Drop world_version_id from captures ------------------------------------
-- All captures were deleted above, so no data loss. Drop the index first.

DROP INDEX IF EXISTS idx_captures_work_lookup;
ALTER TABLE captures DROP COLUMN world_version_id;

-- Recreate the work lookup index without world_version_id.
CREATE INDEX idx_captures_work_lookup
    ON captures(shader_version_id, scene_id, status)
    INCLUDE (profile_id, preset_id)
    WHERE status IN ('completed', 'uploading');

-- 7. Drop world_versions table -----------------------------------------------
-- No FK references remain (captures.world_version_id already dropped).

DROP TABLE IF EXISTS world_versions CASCADE;

-- 8. Drop worlds table -------------------------------------------------------
-- scenes.world_id already dropped; world_versions already dropped.

DROP TABLE IF EXISTS worlds CASCADE;

-- 9. Drop pending_uploads table ----------------------------------------------

DROP TABLE IF EXISTS pending_uploads CASCADE;
