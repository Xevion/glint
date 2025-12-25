--------------------------------------------------------------------------------
-- PENDING_UPLOADS TABLE
-- Tracks world uploads in progress (presigned URL workflow)
--------------------------------------------------------------------------------

CREATE TABLE pending_uploads (
    upload_id TEXT PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    minecraft_version TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    upload_key TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

CREATE INDEX idx_pending_uploads_slug ON pending_uploads(slug);
CREATE INDEX idx_pending_uploads_expires_at ON pending_uploads(expires_at);
