-- Glint SQL Views
--
-- Canonical view definitions. Applied on app startup and after migrations.
-- Views are ordered by dependency (layer 0 -> 1 -> 2 -> 3).
-- Edit THIS file, not migrations, when changing view logic.
--
-- To apply manually: just migrate-run (runs migrations then views)
-- Views are also applied on every app startup via db::apply_views().

-- =============================================================================
-- Layer 1: Latest version lookups
-- =============================================================================

-- Latest world version per world (by created_at DESC, id DESC tiebreaker).
-- Replaces the _lwv / latest_world_versions CTE in capture, health, and work queries.
CREATE OR REPLACE VIEW latest_world_versions AS
SELECT DISTINCT ON (world_id)
    id, world_id, file_url, file_hash, size_bytes, created_at
FROM world_versions
ORDER BY world_id, created_at DESC, id DESC;

-- Latest scene version per scene (by created_at DESC, id DESC tiebreaker).
-- Replaces the _lsv / latest_scene_versions CTE in capture, health, and work queries.
CREATE OR REPLACE VIEW latest_scene_versions AS
SELECT DISTINCT ON (scene_id)
    id, scene_id, x, y, z, pitch, yaw,
    time_of_day_ticks, weather, weather_intensity, moon_phase, biome, created_at
FROM scene_versions
ORDER BY scene_id, created_at DESC, id DESC;

-- Latest shader version per shader (by upstream_published_at DESC, created_at DESC).
-- Replaces the latest_versions CTE in health and work queries.
CREATE OR REPLACE VIEW latest_shader_versions AS
SELECT DISTINCT ON (shader_id)
    id, shader_id, version, capture_failure_count, supported_profiles,
    download_url, file_hash, file_size, game_versions, release_channel,
    modrinth_version_id, curseforge_file_id,
    upstream_published_at, created_at, last_capture_error
FROM shader_versions
ORDER BY shader_id, upstream_published_at DESC NULLS LAST, created_at DESC;

-- =============================================================================
-- Layer 2: Composed views
-- =============================================================================

-- Every (shader_version, scene, profile) triple that should have a capture.
-- Expands shader profiles via JSONB lateral join. Used by health and work queries.
-- Replaces the target_matrix / needed CTE in capture_health.rs and work.rs.
CREATE OR REPLACE VIEW capture_target_matrix AS
-- Branch 1: shader versions WITH profiles
SELECT
    sv.id AS shader_version_id,
    s.id AS scene_id,
    s.world_id,
    p.profile AS profile
FROM latest_shader_versions sv
CROSS JOIN scenes s
CROSS JOIN LATERAL jsonb_array_elements_text(
    CASE
        WHEN sv.supported_profiles IS NOT NULL AND sv.supported_profiles != '[]'
        THEN sv.supported_profiles::jsonb
        ELSE '[]'::jsonb
    END
) AS p(profile)
WHERE s.active = TRUE

UNION ALL

-- Branch 2: shader versions WITHOUT profiles
SELECT
    sv.id AS shader_version_id,
    s.id AS scene_id,
    s.world_id,
    NULL AS profile
FROM latest_shader_versions sv
CROSS JOIN scenes s
WHERE s.active = TRUE
  AND (sv.supported_profiles IS NULL OR sv.supported_profiles = '[]');

-- Captures augmented with a computed freshness column.
-- Freshness is 'superseded' | 'stale' | 'fresh' based on:
--   1. Non-completed/uploading status -> superseded
--   2. A newer capture exists for the same target -> superseded
--   3. World or scene version doesn't match latest -> stale
--   4. Otherwise -> fresh
-- Replaces the CASE statement in capture_ctx_query! and get_detail.
CREATE OR REPLACE VIEW captures_with_freshness AS
SELECT
    c.*,
    CASE
        WHEN c.status NOT IN ('completed', 'uploading') THEN 'superseded'
        WHEN EXISTS (
            SELECT 1 FROM captures c2
            WHERE c2.shader_version_id = c.shader_version_id
              AND c2.scene_id = c.scene_id
              AND c2.profile IS NOT DISTINCT FROM c.profile
              AND c2.status IN ('completed', 'uploading')
              AND c2.captured_at > c.captured_at
        ) THEN 'superseded'
        WHEN (c.world_version_id IS NOT NULL AND c.world_version_id IS DISTINCT FROM lwv.id)
          OR (c.scene_version_id IS NOT NULL AND c.scene_version_id IS DISTINCT FROM lsv.id)
        THEN 'stale'
        ELSE 'fresh'
    END AS freshness
FROM captures c
LEFT JOIN scenes sc ON sc.id = c.scene_id
LEFT JOIN latest_world_versions lwv ON lwv.world_id = sc.world_id
LEFT JOIN latest_scene_versions lsv ON lsv.scene_id = sc.id;

-- =============================================================================
-- Layer 3: Full projections
-- =============================================================================

-- Full capture-with-context projection.
-- Replaces the capture_ctx_query! macro entirely.
-- Column set matches the CaptureWithContext Rust struct plus passthrough columns
-- for downstream views (capture_details) to avoid re-joining captures.
CREATE OR REPLACE VIEW capture_contexts AS
SELECT
    c.id,
    c.scene_id,
    s.slug AS shader_slug,
    s.name AS shader_name,
    sv.version AS shader_version,
    c.profile,
    c.image_path,
    c.image_url,
    c.thumbhash,
    c.captured_at,
    c.resolution_width,
    c.resolution_height,
    c.file_size_bytes,
    cri.run_id,
    cr.status AS run_status,
    (SELECT sa.name FROM shader_authors sa WHERE sa.shader_id = s.id LIMIT 1) AS shader_author,
    sc.name AS scene_name,
    sc.slug AS scene_slug,
    c.freshness,
    -- Extra columns needed by downstream queries for filtering/ordering
    c.shader_version_id,
    c.status AS capture_status,
    sv.shader_id,
    sc.active AS scene_active,
    sc.world_id,
    -- Technical metadata passthrough (avoids re-joining captures in capture_details)
    c.error_message,
    c.video_url,
    c.avg_fps,
    c.min_fps,
    c.max_fps,
    c.frame_time_avg,
    c.frame_time_p99,
    c.minecraft_version,
    c.iris_version,
    c.gpu_model,
    c.content_type,
    c.world_version_id,
    c.scene_version_id,
    c.created_at,
    c.updated_at
FROM captures_with_freshness c
JOIN shader_versions sv ON c.shader_version_id = sv.id
JOIN shaders s ON sv.shader_id = s.id
LEFT JOIN capture_run_items cri ON cri.capture_id = c.id
LEFT JOIN capture_runs cr ON cri.run_id = cr.id
LEFT JOIN scenes sc ON c.scene_id = sc.id;

-- Extended capture projection with full technical metadata.
-- Replaces the CaptureFullRow query in CaptureRepo::get_detail.
-- Column set matches the CaptureDetail Rust struct (minus related captures).
-- All columns sourced from capture_contexts (no extra captures join needed).
CREATE OR REPLACE VIEW capture_details AS
SELECT
    cc.id,
    cc.scene_id,
    cc.shader_slug,
    cc.shader_name,
    cc.shader_version,
    cc.shader_version_id,
    cc.profile,
    cc.image_path,
    cc.image_url,
    cc.thumbhash,
    cc.captured_at,
    cc.resolution_width,
    cc.resolution_height,
    cc.file_size_bytes,
    cc.run_id,
    cc.run_status,
    cc.shader_author,
    cc.scene_name,
    cc.scene_slug,
    cc.freshness,
    -- Technical metadata (passthrough from capture_contexts)
    cc.capture_status AS status,
    cc.error_message,
    cc.video_url,
    cc.avg_fps,
    cc.min_fps,
    cc.max_fps,
    cc.frame_time_avg,
    cc.frame_time_p99,
    cc.minecraft_version,
    cc.iris_version,
    cc.gpu_model,
    cc.content_type,
    cc.world_version_id,
    cc.scene_version_id,
    cc.created_at,
    cc.updated_at,
    -- Extra for filtering
    cc.shader_id,
    cc.scene_active,
    cc.capture_status
FROM capture_contexts cc;
