-- Shader profiles as first-class entities and extraction metadata sidecar.
--
-- Profiles move from a bare TEXT column on captures to a dedicated table with
-- full option maps, labels, and sort ordering. Extraction status tracking goes
-- on shader_versions so background jobs can find unprocessed versions.

-- 1. New tables ---------------------------------------------------------------

CREATE TABLE shader_version_profiles (
    id                TEXT PRIMARY KEY,
    shader_version_id TEXT NOT NULL REFERENCES shader_versions(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    label             TEXT,
    description       TEXT,
    options           JSONB NOT NULL DEFAULT '{}',
    sort_order        INT NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(shader_version_id, name)
);

CREATE TABLE shader_version_metadata (
    shader_version_id      TEXT PRIMARY KEY REFERENCES shader_versions(id) ON DELETE CASCADE,
    pipeline_features      JSONB,
    iris_features_required JSONB,
    iris_features_optional JSONB,
    settings_screen        JSONB,
    file_paths             JSONB,
    dimension_support      JSONB,
    has_custom_textures    BOOLEAN,
    extracted_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. Extraction status on shader_versions ------------------------------------

ALTER TABLE shader_versions
    ADD COLUMN extraction_status TEXT NOT NULL DEFAULT 'pending',
    ADD COLUMN extraction_error  TEXT,
    ADD COLUMN extracted_at      TIMESTAMPTZ;

CREATE INDEX idx_sv_extraction_pending
    ON shader_versions(extraction_status)
    WHERE extraction_status = 'pending';

-- 3. Drop views that depend on columns we're about to remove -----------------
-- views.sql re-applies all views on app startup, so this is safe.

DROP VIEW IF EXISTS capture_details CASCADE;
DROP VIEW IF EXISTS capture_contexts CASCADE;
DROP VIEW IF EXISTS captures_with_freshness CASCADE;
DROP VIEW IF EXISTS latest_shader_versions CASCADE;

-- 4. Migrate captures.profile TEXT → profile_id FK ---------------------------

ALTER TABLE captures
    ADD COLUMN profile_id TEXT REFERENCES shader_version_profiles(id) ON DELETE SET NULL;

-- Backfill: create profile rows from existing capture profile names, then link.
-- This INSERT creates stub profiles (empty options) for any profile names that
-- were recorded on captures before the extraction system existed.
INSERT INTO shader_version_profiles (id, shader_version_id, name, options)
SELECT
    -- Deterministic ID from the (version, profile) pair so re-runs are idempotent
    encode(sha256(convert_to(c.shader_version_id || '/' || c.profile, 'UTF8')), 'hex'),
    c.shader_version_id,
    c.profile,
    '{}'::jsonb
FROM (
    SELECT DISTINCT shader_version_id, profile
    FROM captures
    WHERE profile IS NOT NULL
) c
ON CONFLICT (shader_version_id, name) DO NOTHING;

UPDATE captures c
SET profile_id = svp.id
FROM shader_version_profiles svp
WHERE c.shader_version_id = svp.shader_version_id
  AND c.profile = svp.name;

ALTER TABLE captures DROP COLUMN profile;

CREATE INDEX idx_captures_profile_id
    ON captures(profile_id)
    WHERE profile_id IS NOT NULL;

-- 5. Migrate capture_run_items.profile TEXT → profile_id FK ------------------

ALTER TABLE capture_run_items
    ADD COLUMN profile_id TEXT REFERENCES shader_version_profiles(id) ON DELETE SET NULL;

UPDATE capture_run_items cri
SET profile_id = svp.id
FROM shader_version_profiles svp
WHERE cri.shader_version_id = svp.shader_version_id
  AND cri.profile = svp.name;

ALTER TABLE capture_run_items DROP COLUMN profile;

-- 6. Drop redundant supported_profiles column --------------------------------

ALTER TABLE shader_versions DROP COLUMN supported_profiles;

-- 7. Rebuild covering indexes that included the old profile column -----------
-- These replace idx_captures_freshness_lookup and idx_captures_work_lookup
-- which were dropped when the profile column was removed.

DROP INDEX IF EXISTS idx_captures_freshness_lookup;
DROP INDEX IF EXISTS idx_captures_work_lookup;

CREATE INDEX idx_captures_freshness_lookup
    ON captures(shader_version_id, scene_id, status, captured_at DESC)
    INCLUDE (profile_id)
    WHERE status IN ('completed', 'uploading');

CREATE INDEX idx_captures_work_lookup
    ON captures(shader_version_id, scene_id, status)
    INCLUDE (profile_id, world_version_id)
    WHERE status IN ('completed', 'uploading');
