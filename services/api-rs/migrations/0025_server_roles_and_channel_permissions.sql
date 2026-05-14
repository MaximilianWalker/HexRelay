DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'server_channels_server_channel_unique'
    ) THEN
        ALTER TABLE server_channels
            ADD CONSTRAINT server_channels_server_channel_unique UNIQUE (server_id, channel_id);
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS server_roles (
    role_id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    rank INTEGER NOT NULL DEFAULT 0 CHECK (rank >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT server_roles_server_fk
        FOREIGN KEY (server_id)
        REFERENCES servers(server_id)
        ON DELETE CASCADE,
    CONSTRAINT server_roles_name_non_empty CHECK (BTRIM(name) <> ''),
    CONSTRAINT server_roles_server_name_unique UNIQUE (server_id, name),
    CONSTRAINT server_roles_server_role_unique UNIQUE (server_id, role_id)
);

CREATE INDEX IF NOT EXISTS server_roles_server_rank_idx
    ON server_roles (server_id, rank DESC, name ASC, role_id ASC);

CREATE TABLE IF NOT EXISTS server_membership_roles (
    server_id TEXT NOT NULL,
    identity_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server_id, identity_id, role_id),
    CONSTRAINT server_membership_roles_membership_fk
        FOREIGN KEY (server_id, identity_id)
        REFERENCES server_memberships(server_id, identity_id)
        ON DELETE CASCADE,
    CONSTRAINT server_membership_roles_role_fk
        FOREIGN KEY (server_id, role_id)
        REFERENCES server_roles(server_id, role_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS server_membership_roles_role_idx
    ON server_membership_roles (server_id, role_id, identity_id);

CREATE TABLE IF NOT EXISTS server_channel_role_permissions (
    server_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    can_read BOOLEAN NOT NULL DEFAULT TRUE,
    can_send BOOLEAN NOT NULL DEFAULT FALSE,
    can_manage BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server_id, channel_id, role_id),
    CONSTRAINT server_channel_role_permissions_channel_fk
        FOREIGN KEY (server_id, channel_id)
        REFERENCES server_channels(server_id, channel_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channel_role_permissions_role_fk
        FOREIGN KEY (server_id, role_id)
        REFERENCES server_roles(server_id, role_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channel_role_permissions_send_requires_read CHECK (can_read OR NOT can_send),
    CONSTRAINT server_channel_role_permissions_manage_requires_read CHECK (can_read OR NOT can_manage)
);

CREATE INDEX IF NOT EXISTS server_channel_role_permissions_role_idx
    ON server_channel_role_permissions (server_id, role_id, channel_id);
