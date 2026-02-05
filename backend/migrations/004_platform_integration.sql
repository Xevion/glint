-- Platform integration: extend shaders/versions for Modrinth & CurseForge adoption

ALTER TABLE shaders
    ADD COLUMN icon_url TEXT,
    ADD COLUMN source_url TEXT,
    ADD COLUMN license_id TEXT,
    ADD COLUMN upstream_downloads BIGINT DEFAULT 0,
    ADD COLUMN upstream_updated_at TIMESTAMPTZ,
    ADD COLUMN last_synced_at TIMESTAMPTZ;

ALTER TABLE shader_versions
    ADD COLUMN curseforge_file_id INTEGER,
    ADD COLUMN file_size BIGINT,
    ADD COLUMN game_versions TEXT,
    ADD COLUMN release_channel TEXT DEFAULT 'release',
    ADD COLUMN upstream_published_at TIMESTAMPTZ;

CREATE TABLE shader_authors (
    id TEXT PRIMARY KEY,
    shader_id TEXT NOT NULL REFERENCES shaders(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    url TEXT,
    platform TEXT NOT NULL,
    UNIQUE(shader_id, name, platform)
);
