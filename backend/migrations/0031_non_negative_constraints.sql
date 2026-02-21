-- Non-negative and range CHECK constraints for semantically positive columns.
-- These enforce storage-layer guarantees without changing application types.
-- sort_order fields are intentionally excluded (negative values reserved for future ordering semantics).

ALTER TABLE captures
  ADD CONSTRAINT captures_resolution_width_positive CHECK (resolution_width > 0),
  ADD CONSTRAINT captures_resolution_height_positive CHECK (resolution_height > 0),
  ADD CONSTRAINT captures_file_size_bytes_non_negative CHECK (file_size_bytes >= 0);

ALTER TABLE capture_runs
  ADD CONSTRAINT capture_runs_resolution_width_positive CHECK (resolution_width > 0),
  ADD CONSTRAINT capture_runs_resolution_height_positive CHECK (resolution_height > 0),
  ADD CONSTRAINT capture_runs_total_items_non_negative CHECK (total_items >= 0),
  ADD CONSTRAINT capture_runs_completed_items_non_negative CHECK (completed_items >= 0),
  ADD CONSTRAINT capture_runs_failed_items_non_negative CHECK (failed_items >= 0),
  ADD CONSTRAINT capture_runs_skipped_items_non_negative CHECK (skipped_items >= 0);

ALTER TABLE capture_run_items
  ADD CONSTRAINT capture_run_items_duration_ms_non_negative CHECK (duration_ms >= 0);

ALTER TABLE backgrounds
  ADD CONSTRAINT backgrounds_width_positive CHECK (width > 0),
  ADD CONSTRAINT backgrounds_height_positive CHECK (height > 0),
  ADD CONSTRAINT backgrounds_file_size_bytes_non_negative CHECK (file_size_bytes >= 0);

ALTER TABLE scene_versions
  ADD CONSTRAINT scene_versions_fov_range CHECK (fov BETWEEN 1 AND 179),
  ADD CONSTRAINT scene_versions_render_distance_positive CHECK (render_distance > 0),
  ADD CONSTRAINT scene_versions_time_of_day_range CHECK (time_of_day_ticks BETWEEN 0 AND 24000),
  ADD CONSTRAINT scene_versions_moon_phase_range CHECK (moon_phase BETWEEN 0 AND 7),
  ADD CONSTRAINT scene_versions_package_size_bytes_non_negative CHECK (package_size_bytes >= 0);

ALTER TABLE scene_presets
  ADD CONSTRAINT scene_presets_time_of_day_range CHECK (time_of_day_ticks BETWEEN 0 AND 24000),
  ADD CONSTRAINT scene_presets_moon_phase_range CHECK (moon_phase BETWEEN 0 AND 7);

ALTER TABLE pending_scene_uploads
  ADD CONSTRAINT pending_scene_uploads_size_bytes_positive CHECK (size_bytes > 0);

ALTER TABLE shader_versions
  ADD CONSTRAINT shader_versions_file_size_non_negative CHECK (file_size >= 0),
  ADD CONSTRAINT shader_versions_capture_failure_count_non_negative CHECK (capture_failure_count >= 0);

ALTER TABLE shaders
  ADD CONSTRAINT shaders_upstream_downloads_non_negative CHECK (upstream_downloads >= 0),
  ADD CONSTRAINT shaders_view_count_non_negative CHECK (view_count >= 0);
