CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    identity_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS sessions_identity_active_idx
ON sessions (identity_id, expires_at)
WHERE revoked_at IS NULL;
