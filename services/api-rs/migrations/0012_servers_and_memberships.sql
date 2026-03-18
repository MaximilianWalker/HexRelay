CREATE TABLE IF NOT EXISTS servers (
    server_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS server_memberships (
    server_id TEXT NOT NULL,
    identity_id TEXT NOT NULL,
    favorite BOOLEAN NOT NULL DEFAULT FALSE,
    muted BOOLEAN NOT NULL DEFAULT FALSE,
    unread_count INTEGER NOT NULL DEFAULT 0 CHECK (unread_count >= 0),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server_id, identity_id),
    CONSTRAINT server_memberships_server_fk
        FOREIGN KEY (server_id)
        REFERENCES servers(server_id)
        ON DELETE CASCADE,
    CONSTRAINT server_memberships_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS server_memberships_identity_joined_idx
    ON server_memberships (identity_id, favorite DESC, joined_at ASC);
