--------------------------------------------------------------------------------
-- AUTHENTICATION TABLES
--------------------------------------------------------------------------------

-- Discord OAuth users
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    discord_id TEXT UNIQUE NOT NULL,
    discord_username TEXT NOT NULL,
    discord_avatar TEXT,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Database-backed sessions (with in-memory cache at runtime)
CREATE TABLE sessions (
    token TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

--------------------------------------------------------------------------------
-- SHADER CATEGORIZATION TABLES
--------------------------------------------------------------------------------

-- Style categories (realistic, fantasy, cartoon, etc.)
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE shader_categories (
    shader_id TEXT NOT NULL REFERENCES shaders(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (shader_id, category_id)
);

CREATE INDEX idx_shader_categories_category ON shader_categories(category_id);

-- Technical features (volumetric, PBR, ray tracing, etc.)
CREATE TABLE features (
    id SERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE shader_features (
    shader_id TEXT NOT NULL REFERENCES shaders(id) ON DELETE CASCADE,
    feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    PRIMARY KEY (shader_id, feature_id)
);

CREATE INDEX idx_shader_features_feature ON shader_features(feature_id);

--------------------------------------------------------------------------------
-- SCENE TAGGING TABLES
--------------------------------------------------------------------------------

-- Scene tags (indoor, sunset, water, etc.)
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE scene_tags (
    scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (scene_id, tag_id)
);

CREATE INDEX idx_scene_tags_tag ON scene_tags(tag_id);
