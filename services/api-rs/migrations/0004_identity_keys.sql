CREATE TABLE IF NOT EXISTS identity_keys (
    identity_id TEXT PRIMARY KEY,
    public_key TEXT NOT NULL,
    algorithm TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
