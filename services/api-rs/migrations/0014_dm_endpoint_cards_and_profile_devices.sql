CREATE TABLE IF NOT EXISTS dm_endpoint_cards (
    identity_id TEXT NOT NULL,
    endpoint_id TEXT NOT NULL,
    endpoint_hint TEXT NOT NULL,
    estimated_rtt_ms INTEGER NOT NULL CHECK (estimated_rtt_ms >= 0),
    priority SMALLINT NOT NULL DEFAULT 0 CHECK (priority >= 0),
    expires_at_epoch BIGINT NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (identity_id, endpoint_id),
    CONSTRAINT dm_endpoint_cards_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS dm_endpoint_cards_identity_expiry_idx
    ON dm_endpoint_cards (identity_id, expires_at_epoch DESC);

CREATE TABLE IF NOT EXISTS dm_profile_devices (
    identity_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    active BOOLEAN NOT NULL,
    last_seen_epoch BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (identity_id, device_id),
    CONSTRAINT dm_profile_devices_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS dm_profile_devices_identity_seen_idx
    ON dm_profile_devices (identity_id, last_seen_epoch DESC);
