-- 005_capture_redesign.sql
-- Capture Architecture Redesign:
-- 1. Drop jobs table (replaced by on-demand work computation)
-- 2. Drop UNIQUE constraint on captures (switch to append-only history)
-- 3. Add capture_runs and capture_run_items tables (audit logging)
-- 4. Add error tracking fields to shader_versions

-- 1. Drop jobs table
DROP TABLE IF EXISTS jobs;

-- 2. Drop UNIQUE constraint on captures to allow append-only history
-- The constraint name from 001_initial.sql
ALTER TABLE captures DROP CONSTRAINT IF EXISTS captures_shader_version_id_scene_id_profile_key;

-- 3. Add capture_runs table
CREATE TABLE capture_runs (
    id TEXT PRIMARY KEY,
    agent_id TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'running',
    total_items INTEGER NOT NULL DEFAULT 0,
    completed_items INTEGER NOT NULL DEFAULT 0,
    failed_items INTEGER NOT NULL DEFAULT 0,
    skipped_items INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT
);

-- 4. Add capture_run_items table
CREATE TABLE capture_run_items (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES capture_runs(id) ON DELETE CASCADE,
    shader_version_id TEXT NOT NULL REFERENCES shader_versions(id) ON DELETE CASCADE,
    scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
    profile TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    capture_id TEXT REFERENCES captures(id) ON DELETE SET NULL,
    error_message TEXT,
    error_log TEXT,
    duration_ms INTEGER,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_capture_run_items_run_id ON capture_run_items(run_id);
CREATE INDEX idx_capture_run_items_status ON capture_run_items(status);
CREATE INDEX idx_capture_runs_status ON capture_runs(status);

-- 5. Add error tracking to shader_versions
ALTER TABLE shader_versions ADD COLUMN capture_failure_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE shader_versions ADD COLUMN last_capture_error TEXT;
