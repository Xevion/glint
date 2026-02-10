-- Composite index for the work generation query's NOT EXISTS subqueries.
-- Covers lookups by (shader_version_id, scene_id, status) with profile and
-- world_version_id as payload columns, avoiding heap fetches.
CREATE INDEX idx_captures_work_lookup
    ON captures (shader_version_id, scene_id, status)
    INCLUDE (profile, world_version_id)
    WHERE status IN ('completed', 'uploading');

-- Composite index for the ORDER BY EXISTS(...) subquery that checks whether
-- a shader_version has any completed captures.
CREATE INDEX idx_captures_version_completed
    ON captures (shader_version_id)
    WHERE status = 'completed';
