--------------------------------------------------------------------------------
-- SCENE SYNC UPDATES
-- Add active/outdated tracking and world-scoped uniqueness
--------------------------------------------------------------------------------

-- Add active flag to scenes (1 = active, 0 = disabled)
ALTER TABLE scenes ADD COLUMN active INTEGER NOT NULL DEFAULT 1;

-- Add outdated flag to captures (1 = outdated, 0 = current)
ALTER TABLE captures ADD COLUMN outdated INTEGER NOT NULL DEFAULT 0;

-- Change slug uniqueness from global to world-scoped
-- Only enforce uniqueness for active scenes (allows slug reuse after soft delete)
DROP INDEX idx_scenes_slug;
CREATE UNIQUE INDEX idx_scenes_world_slug ON scenes(world_id, slug) WHERE active = 1;

-- Add indexes for filtering
CREATE INDEX idx_scenes_active ON scenes(active);
CREATE INDEX idx_captures_outdated ON captures(outdated);
