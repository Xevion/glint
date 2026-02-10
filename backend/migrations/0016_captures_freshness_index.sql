-- Covering index for the captures_with_freshness view's EXISTS subquery.
-- The subquery checks for newer captures with the same target (shader_version_id,
-- scene_id, profile) that are completed/uploading and have a later captured_at.
-- Including captured_at in the key and profile in INCLUDE avoids heap fetches.
CREATE INDEX idx_captures_freshness_lookup
    ON captures (shader_version_id, scene_id, status, captured_at DESC)
    INCLUDE (profile)
    WHERE status IN ('completed', 'uploading');
