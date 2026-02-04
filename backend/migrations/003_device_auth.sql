--------------------------------------------------------------------------------
-- DEVICE AUTHORIZATION FLOW
-- OAuth 2.0 Device Authorization Grant (RFC 8628) for Minecraft mod authentication
--------------------------------------------------------------------------------

-- Track session source (web browser vs device/mod)
ALTER TABLE sessions ADD COLUMN source TEXT NOT NULL DEFAULT 'web';

-- Device authorization codes for mod authentication
CREATE TABLE device_codes (
    -- Random 32-char hex secret used by mod for polling
    device_code TEXT PRIMARY KEY,
    -- User-facing code in GLINT-XXXXXX format
    user_code TEXT UNIQUE NOT NULL,
    -- Set when user authorizes the device
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    -- pending -> authorized -> used
    status TEXT NOT NULL DEFAULT 'pending',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_device_codes_user_code ON device_codes(user_code);
CREATE INDEX idx_device_codes_expires_at ON device_codes(expires_at);
