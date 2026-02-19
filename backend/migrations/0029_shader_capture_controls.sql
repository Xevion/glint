-- Add manual capture controls to shaders:
-- - preferred_version_id: pin a specific version for capture targeting and frontend defaults
-- - capture_enabled: toggle to pause/resume capture scheduling for a shader

ALTER TABLE shaders
ADD COLUMN preferred_version_id TEXT REFERENCES shader_versions(id) ON DELETE SET NULL,
ADD COLUMN capture_enabled BOOLEAN NOT NULL DEFAULT TRUE;
