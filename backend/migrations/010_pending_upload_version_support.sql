-- Extend pending_uploads to support world version uploads (not just world creation).
-- Version uploads set world_id; world creation uploads set slug/name/minecraft_version.

ALTER TABLE pending_uploads
    ADD COLUMN world_id TEXT REFERENCES worlds(id) ON DELETE CASCADE,
    ALTER COLUMN slug DROP NOT NULL,
    ALTER COLUMN name DROP NOT NULL,
    ALTER COLUMN minecraft_version DROP NOT NULL;

CREATE INDEX idx_pending_uploads_world_id ON pending_uploads(world_id)
    WHERE world_id IS NOT NULL;
