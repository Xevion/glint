-- Denormalized view count on shaders, maintained by trigger.
-- Eliminates per-request GROUP BY on shader_views for list/detail endpoints.
ALTER TABLE shaders ADD COLUMN view_count BIGINT NOT NULL DEFAULT 0;

-- Backfill from existing data
UPDATE shaders SET view_count = COALESCE(
    (SELECT COUNT(*) FROM shader_views WHERE shader_views.shader_id = shaders.id),
    0
);

-- Increment on successful insert (ON CONFLICT DO NOTHING = no insert = no trigger fire)
CREATE FUNCTION increment_shader_view_count() RETURNS TRIGGER AS $$
BEGIN
    UPDATE shaders SET view_count = view_count + 1 WHERE id = NEW.shader_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_shader_view_count
    AFTER INSERT ON shader_views
    FOR EACH ROW
    EXECUTE FUNCTION increment_shader_view_count();
