ALTER TABLE local_server
    ADD COLUMN IF NOT EXISTS description TEXT NOT NULL DEFAULT '';

DROP INDEX IF EXISTS server_memberships_joined_idx;
CREATE INDEX IF NOT EXISTS server_memberships_joined_idx
    ON server_memberships (pinned DESC, joined_at ASC, identity_id ASC);

ALTER TABLE server_channels DROP CONSTRAINT IF EXISTS server_channels_kind_check;
ALTER TABLE server_channels
    ADD CONSTRAINT server_channels_kind_check CHECK (kind IN ('text', 'voice'));

CREATE TABLE IF NOT EXISTS server_administrators (
    identity_id TEXT PRIMARY KEY,
    is_owner BOOLEAN NOT NULL DEFAULT FALSE,
    is_admin BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT server_administrators_membership_fk
        FOREIGN KEY (identity_id)
        REFERENCES server_memberships(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT server_administrators_scope_check CHECK (is_owner OR is_admin)
);

CREATE TABLE IF NOT EXISTS server_bootstrap_credentials (
    credential_id TEXT PRIMARY KEY,
    credential_secret_hash TEXT NOT NULL UNIQUE,
    created_by_identity_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT server_bootstrap_credentials_creator_fk
        FOREIGN KEY (created_by_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS contact_preferences (
    owner_identity_id TEXT NOT NULL,
    contact_identity_id TEXT NOT NULL,
    pinned BOOLEAN NOT NULL DEFAULT FALSE,
    muted BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (owner_identity_id, contact_identity_id),
    CONSTRAINT contact_preferences_owner_fk
        FOREIGN KEY (owner_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT contact_preferences_contact_fk
        FOREIGN KEY (contact_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT contact_preferences_distinct_identity_check
        CHECK (owner_identity_id <> contact_identity_id)
);

CREATE INDEX IF NOT EXISTS contact_preferences_owner_pinned_idx
    ON contact_preferences (owner_identity_id, pinned DESC, updated_at DESC);

CREATE TABLE IF NOT EXISTS user_blocks (
    blocker_identity_id TEXT NOT NULL,
    blocked_identity_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (blocker_identity_id, blocked_identity_id),
    CONSTRAINT user_blocks_blocker_fk
        FOREIGN KEY (blocker_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT user_blocks_blocked_fk
        FOREIGN KEY (blocked_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT user_blocks_distinct_identity_check
        CHECK (blocker_identity_id <> blocked_identity_id)
);

CREATE INDEX IF NOT EXISTS user_blocks_blocked_idx
    ON user_blocks (blocked_identity_id, blocker_identity_id);

CREATE TABLE IF NOT EXISTS user_mutes (
    muter_identity_id TEXT NOT NULL,
    muted_identity_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (muter_identity_id, muted_identity_id),
    CONSTRAINT user_mutes_muter_fk
        FOREIGN KEY (muter_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT user_mutes_muted_fk
        FOREIGN KEY (muted_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT user_mutes_distinct_identity_check
        CHECK (muter_identity_id <> muted_identity_id)
);

CREATE INDEX IF NOT EXISTS user_mutes_muted_idx
    ON user_mutes (muted_identity_id, muter_identity_id);
