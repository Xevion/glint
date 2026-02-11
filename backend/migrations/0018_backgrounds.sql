-- Runtime-configurable page backgrounds (replaces build-time wallpaper system).
-- Images are uploaded to R2 and served via Cloudflare Image Transforms.

CREATE TABLE backgrounds (
    id TEXT PRIMARY KEY,
    image_url TEXT NOT NULL,
    image_path TEXT NOT NULL,
    thumbhash TEXT,
    theme_mode TEXT NOT NULL DEFAULT 'both',
    enabled BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    original_filename TEXT,
    width INTEGER,
    height INTEGER,
    file_size_bytes BIGINT,
    content_type TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for the public query (enabled backgrounds, ordered)
CREATE INDEX idx_backgrounds_enabled ON backgrounds (enabled, sort_order);
