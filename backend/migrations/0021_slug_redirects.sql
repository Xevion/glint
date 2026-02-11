-- Track old slugs for 301 redirects when entities are renamed.
CREATE TABLE slug_redirects (
    id SERIAL PRIMARY KEY,
    entity_type TEXT NOT NULL,  -- 'shader', 'scene', 'world'
    old_slug TEXT NOT NULL,
    entity_id TEXT NOT NULL,    -- the entity's current ID (FK not enforced — entity may be deleted)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Fast lookup: given an entity type and old slug, find the entity
CREATE UNIQUE INDEX idx_slug_redirects_lookup
    ON slug_redirects(entity_type, old_slug);
