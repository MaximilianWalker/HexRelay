CREATE TABLE IF NOT EXISTS invites (
    token TEXT PRIMARY KEY,
    mode TEXT NOT NULL,
    node_fingerprint TEXT NOT NULL,
    expires_at TIMESTAMPTZ,
    max_uses INTEGER,
    uses INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
