-- Scene uploads: track pending scene package uploads and record minecraft version.
--
-- This migration:
-- 1. Adds minecraft_version column to scene_versions
-- 2. Creates pending_scene_uploads table for the initiate/confirm upload flow

-- 1. Add minecraft_version to scene_versions --------------------------------

ALTER TABLE scene_versions ADD COLUMN minecraft_version TEXT;

-- 2. Create pending_scene_uploads table --------------------------------------
-- Tracks in-flight scene package uploads between the initiate and confirm steps.
-- For new scenes, scene_id is NULL and the scene_* metadata columns are populated.
-- For existing scenes, scene_id is set and the metadata columns are NULL.

CREATE TABLE pending_scene_uploads (
    id                TEXT PRIMARY KEY,
    -- Null for new scenes (scene doesn't exist yet), set for existing scenes
    scene_id          TEXT REFERENCES scenes(id) ON DELETE CASCADE,
    -- Only populated for new scenes (scene metadata to create on confirm)
    scene_name        TEXT,
    scene_slug        TEXT,
    scene_dimension   TEXT,
    scene_description TEXT,
    -- Upload metadata
    minecraft_version TEXT NOT NULL,
    file_hash         TEXT NOT NULL,
    size_bytes        BIGINT NOT NULL,
    r2_key            TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at        TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '1 hour')
);

CREATE INDEX idx_pending_scene_uploads_expires ON pending_scene_uploads(expires_at);
