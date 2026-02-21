-- Add slug column to shader_authors, populated from name.
-- Slug logic mirrors backend/src/slug.rs: lowercase, strip apostrophes,
-- replace non-alphanumeric with hyphens, collapse, trim.

ALTER TABLE shader_authors ADD COLUMN slug TEXT NOT NULL DEFAULT '';

UPDATE shader_authors SET slug = TRIM(BOTH '-' FROM regexp_replace(
    regexp_replace(lower(replace(name, '''', '')), '[^a-z0-9]+', '-', 'g'),
    '-+', '-', 'g'
));

CREATE INDEX idx_shader_authors_slug ON shader_authors (slug);
