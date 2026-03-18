CREATE TABLE IF NOT EXISTS dm_fanout_stream_heads (
    identity_id TEXT PRIMARY KEY,
    latest_cursor BIGINT NOT NULL DEFAULT 0 CHECK (latest_cursor >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT dm_fanout_stream_heads_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dm_fanout_device_cursors (
    identity_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    cursor BIGINT NOT NULL DEFAULT 0 CHECK (cursor >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (identity_id, device_id),
    CONSTRAINT dm_fanout_device_cursors_device_fk
        FOREIGN KEY (identity_id, device_id)
        REFERENCES dm_profile_devices(identity_id, device_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS dm_fanout_device_cursors_identity_idx
    ON dm_fanout_device_cursors (identity_id, cursor DESC);
