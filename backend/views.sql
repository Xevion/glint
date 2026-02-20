-- Glint SQL Views
--
-- Canonical view definitions. Applied on app startup and after migrations.
-- Views are ordered by dependency (layer 0 -> 1 -> 2 -> 3).
-- Edit THIS file, not migrations, when changing view logic.
--
-- To apply manually: just migrate-run (runs migrations then views)
-- Views are also applied on every app startup via db::apply_views().

-- Layer 1: Latest version lookups

-- Latest scene version per scene (by created_at DESC, id DESC tiebreaker).
-- Includes package and rendering fields from the scene-packages migration.
-- Drop and recreate to accommodate new minecraft_version column.
DROP VIEW IF EXISTS latest_scene_versions CASCADE;
CREATE OR REPLACE VIEW latest_scene_versions AS
SELECT DISTINCT ON (scene_id)
    id, scene_id, x, y, z, pitch, yaw,
    time_of_day_ticks, weather, weather_intensity, moon_phase, biome,
    fov, render_distance,
    package_url, package_hash, package_size_bytes,
    created_at, minecraft_version
FROM scene_versions
ORDER BY scene_id, created_at DESC, id DESC;

-- Latest shader version per shader (by upstream_published_at DESC, created_at DESC).
-- Respects shaders.preferred_version_id: if set, that version sorts first.
-- Replaces the latest_versions CTE in health and work queries.
DROP VIEW IF EXISTS latest_shader_versions CASCADE;
CREATE OR REPLACE VIEW latest_shader_versions AS
SELECT DISTINCT ON (sv.shader_id)
    sv.id, sv.shader_id, sv.version, sv.capture_failure_count,
    sv.download_url, sv.file_hash, sv.file_size, sv.game_versions, sv.release_channel,
    sv.modrinth_version_id, sv.curseforge_file_id,
    sv.upstream_published_at, sv.created_at, sv.last_capture_error,
    sv.extraction_status, sv.extraction_error, sv.extracted_at
FROM shader_versions sv
JOIN shaders s ON s.id = sv.shader_id
ORDER BY sv.shader_id,
    (sv.id = s.preferred_version_id) DESC,
    sv.upstream_published_at DESC NULLS LAST,
    sv.created_at DESC;

-- Layer 2: Composed views

-- Every (shader_version, scene, preset, profile) tuple that should have a capture.
-- Cross-joins scenes × presets × shader versions. Expands shader profiles via
-- the shader_version_profiles table. Used by health and work queries.
-- Filters out shaders with capture_enabled = FALSE.
CREATE OR REPLACE VIEW capture_target_matrix AS
-- Branch 1: shader versions WITH profiles
SELECT
    sv.id  AS shader_version_id,
    s.id   AS scene_id,
    lsv.id AS scene_version_id,
    sp.id  AS preset_id,
    svp.id AS profile_id
FROM latest_shader_versions sv
JOIN shaders sh ON sh.id = sv.shader_id
JOIN shader_version_profiles svp ON svp.shader_version_id = sv.id
CROSS JOIN scenes s
JOIN latest_scene_versions lsv ON lsv.scene_id = s.id
JOIN scene_presets sp ON sp.scene_id = s.id
WHERE s.active = TRUE
  AND sh.capture_enabled = TRUE

UNION ALL

-- Branch 2: shader versions WITHOUT profiles
SELECT
    sv.id  AS shader_version_id,
    s.id   AS scene_id,
    lsv.id AS scene_version_id,
    sp.id  AS preset_id,
    NULL   AS profile_id
FROM latest_shader_versions sv
JOIN shaders sh ON sh.id = sv.shader_id
CROSS JOIN scenes s
JOIN latest_scene_versions lsv ON lsv.scene_id = s.id
JOIN scene_presets sp ON sp.scene_id = s.id
WHERE s.active = TRUE
  AND sh.capture_enabled = TRUE
  AND NOT EXISTS (
      SELECT 1 FROM shader_version_profiles svp
      WHERE svp.shader_version_id = sv.id
  );

-- Captures augmented with a computed freshness column.
-- Freshness is 'superseded' | 'stale' | 'fresh' based on:
--   1. Non-completed/uploading status -> superseded
--   2. A newer capture exists for the same target (including preset_id) -> superseded
--   3. Scene version doesn't match latest -> stale
--   4. Capture was taken before preset was edited -> stale
--   5. Otherwise -> fresh
CREATE OR REPLACE VIEW captures_with_freshness AS
SELECT
    c.*,
    CASE
        WHEN c.status NOT IN ('completed', 'uploading') THEN 'superseded'
        WHEN EXISTS (
            SELECT 1 FROM captures c2
            WHERE c2.shader_version_id = c.shader_version_id
              AND c2.scene_id = c.scene_id
              AND c2.preset_id IS NOT DISTINCT FROM c.preset_id
              AND c2.profile_id IS NOT DISTINCT FROM c.profile_id
              AND c2.status IN ('completed', 'uploading')
              AND c2.captured_at > c.captured_at
        ) THEN 'superseded'
        WHEN c.scene_version_id IS DISTINCT FROM lsv.id
        THEN 'stale'
        WHEN c.preset_id IS NOT NULL
             AND c.captured_at < sp.updated_at
        THEN 'stale'
        ELSE 'fresh'
    END AS freshness
FROM captures c
LEFT JOIN latest_scene_versions lsv ON lsv.scene_id = c.scene_id
LEFT JOIN scene_presets sp ON sp.id = c.preset_id;

-- Layer 3: Full projections

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
    c.profile_id,
    svp.name AS profile_name,
    svp.display_name AS profile_display_name,
    c.preset_id,
    sp.name AS preset_name,
    sp.slug AS preset_slug,
    c.image_path,
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
    c.scene_version_id,
    c.analysis,
    c.created_at,
    c.updated_at
FROM captures_with_freshness c
JOIN shader_versions sv ON c.shader_version_id = sv.id
JOIN shaders s ON sv.shader_id = s.id
LEFT JOIN shader_version_profiles svp ON c.profile_id = svp.id
LEFT JOIN scene_presets sp ON c.preset_id = sp.id
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
    cc.profile_id,
    cc.profile_name,
    cc.profile_display_name,
    cc.preset_id,
    cc.preset_name,
    cc.preset_slug,
    cc.image_path,
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
    cc.scene_version_id,
    cc.analysis,
    cc.created_at,
    cc.updated_at,
    -- Extra for filtering
    cc.shader_id,
    cc.scene_active
FROM capture_contexts cc;
