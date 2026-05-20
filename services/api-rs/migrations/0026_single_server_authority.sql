DROP TABLE IF EXISTS server_channel_role_permissions CASCADE;
DROP TABLE IF EXISTS server_membership_roles CASCADE;
DROP TABLE IF EXISTS server_roles CASCADE;
DROP TABLE IF EXISTS server_channel_message_mentions CASCADE;
DROP TABLE IF EXISTS server_channel_messages CASCADE;
DROP TABLE IF EXISTS server_channels CASCADE;
DROP TABLE IF EXISTS server_memberships CASCADE;
DROP TABLE IF EXISTS servers CASCADE;

CREATE TABLE local_server (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO local_server (singleton, name)
VALUES (TRUE, 'Local Server');

CREATE TABLE server_memberships (
    identity_id TEXT PRIMARY KEY,
    pinned BOOLEAN NOT NULL DEFAULT FALSE,
    muted BOOLEAN NOT NULL DEFAULT FALSE,
    unread_count INTEGER NOT NULL DEFAULT 0 CHECK (unread_count >= 0),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT server_memberships_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE INDEX server_memberships_joined_idx
    ON server_memberships (pinned DESC, joined_at ASC, identity_id ASC);

CREATE TABLE server_channels (
    channel_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'text',
    last_message_seq BIGINT NOT NULL DEFAULT 0 CHECK (last_message_seq >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT server_channels_kind_check CHECK (kind IN ('text'))
);

CREATE INDEX server_channels_created_idx
    ON server_channels (created_at ASC, channel_id ASC);

CREATE TABLE server_channel_messages (
    message_id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    channel_seq BIGINT NOT NULL CHECK (channel_seq > 0),
    content TEXT NOT NULL,
    reply_to_message_id TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    edited_at TIMESTAMPTZ NULL,
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT server_channel_messages_channel_fk
        FOREIGN KEY (channel_id)
        REFERENCES server_channels(channel_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channel_messages_author_fk
        FOREIGN KEY (author_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channel_messages_reply_fk
        FOREIGN KEY (reply_to_message_id)
        REFERENCES server_channel_messages(message_id)
        ON DELETE SET NULL,
    CONSTRAINT server_channel_messages_channel_seq_unique UNIQUE (channel_id, channel_seq)
);

CREATE INDEX server_channel_messages_channel_seq_idx
    ON server_channel_messages (channel_id, channel_seq DESC);

CREATE INDEX server_channel_messages_reply_idx
    ON server_channel_messages (reply_to_message_id);

CREATE TABLE server_channel_message_mentions (
    message_id TEXT NOT NULL,
    mentioned_identity_id TEXT NOT NULL,
    PRIMARY KEY (message_id, mentioned_identity_id),
    CONSTRAINT server_channel_message_mentions_message_fk
        FOREIGN KEY (message_id)
        REFERENCES server_channel_messages(message_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channel_message_mentions_identity_fk
        FOREIGN KEY (mentioned_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE INDEX server_channel_message_mentions_identity_idx
    ON server_channel_message_mentions (mentioned_identity_id, message_id);

CREATE TABLE server_roles (
    role_id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    rank INTEGER NOT NULL DEFAULT 0 CHECK (rank >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT server_roles_name_non_empty CHECK (BTRIM(name) <> '')
);

CREATE INDEX server_roles_rank_idx
    ON server_roles (rank DESC, name ASC, role_id ASC);

CREATE TABLE server_membership_roles (
    identity_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (identity_id, role_id),
    CONSTRAINT server_membership_roles_membership_fk
        FOREIGN KEY (identity_id)
        REFERENCES server_memberships(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT server_membership_roles_role_fk
        FOREIGN KEY (role_id)
        REFERENCES server_roles(role_id)
        ON DELETE CASCADE
);

CREATE INDEX server_membership_roles_role_idx
    ON server_membership_roles (role_id, identity_id);

CREATE TABLE server_channel_role_permissions (
    channel_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    can_read BOOLEAN NOT NULL DEFAULT TRUE,
    can_send BOOLEAN NOT NULL DEFAULT FALSE,
    can_manage BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (channel_id, role_id),
    CONSTRAINT server_channel_role_permissions_channel_fk
        FOREIGN KEY (channel_id)
        REFERENCES server_channels(channel_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channel_role_permissions_role_fk
        FOREIGN KEY (role_id)
        REFERENCES server_roles(role_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channel_role_permissions_send_requires_read CHECK (can_read OR NOT can_send),
    CONSTRAINT server_channel_role_permissions_manage_requires_read CHECK (can_read OR NOT can_manage)
);

CREATE INDEX server_channel_role_permissions_role_idx
    ON server_channel_role_permissions (role_id, channel_id);
