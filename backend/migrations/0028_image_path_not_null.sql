-- Backfill NULL image_path values from the deterministic R2 key pattern,
-- then enforce NOT NULL so the path is always set at insert time.
--
-- The key pattern is: captures/{shader_id}/{scene_id}/{capture_id}.webp
-- shader_id comes from shader_versions → shaders join.

-- Views depend on captures.* so must be dropped first.
-- They are recreated on app startup via db::apply_views().
DROP VIEW IF EXISTS capture_details CASCADE;
DROP VIEW IF EXISTS capture_contexts CASCADE;
DROP VIEW IF EXISTS captures_with_freshness CASCADE;

UPDATE captures c
SET image_path = 'captures/' || sv.shader_id || '/' || c.scene_id || '/' || c.id || '.webp'
FROM shader_versions sv
WHERE sv.id = c.shader_version_id
  AND c.image_path IS NULL;

ALTER TABLE captures ALTER COLUMN image_path SET NOT NULL;
