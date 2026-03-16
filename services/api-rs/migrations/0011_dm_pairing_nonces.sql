CREATE TABLE IF NOT EXISTS dm_pairing_nonces (
    nonce TEXT PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dm_pairing_nonces_expires_at
    ON dm_pairing_nonces (expires_at);
