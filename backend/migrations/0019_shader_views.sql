-- Shader view tracking with hourly per-viewer deduplication.
-- viewer_hash is a truncated SHA-256 of (client_ip || user_agent), computed server-side.
-- viewed_at is TIMESTAMP (not TIMESTAMPTZ) so date_trunc is immutable for index use.
-- All values are stored as UTC.
CREATE TABLE shader_views (
    id BIGSERIAL PRIMARY KEY,
    shader_id TEXT NOT NULL REFERENCES shaders(id) ON DELETE CASCADE,
    viewer_hash TEXT NOT NULL,
    viewed_at TIMESTAMP NOT NULL DEFAULT (now() AT TIME ZONE 'UTC')
);

-- Hourly dedup: same viewer can only register one view per shader per hour.
-- INSERT ... ON CONFLICT DO NOTHING makes duplicates a no-op.
CREATE UNIQUE INDEX idx_shader_views_dedup
    ON shader_views (shader_id, viewer_hash, date_trunc('hour', viewed_at));

-- Query support: view counts per shader, trending (recent views sorted by count).
CREATE INDEX idx_shader_views_shader_time
    ON shader_views (shader_id, viewed_at);

-- Trending queries scan recent views grouped by shader_id.
CREATE INDEX idx_shader_views_time_shader
    ON shader_views (viewed_at, shader_id);
