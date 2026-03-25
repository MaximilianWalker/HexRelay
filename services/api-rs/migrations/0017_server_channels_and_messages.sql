CREATE TABLE IF NOT EXISTS server_channels (
    channel_id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'text',
    last_message_seq BIGINT NOT NULL DEFAULT 0 CHECK (last_message_seq >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT server_channels_server_fk
        FOREIGN KEY (server_id)
        REFERENCES servers(server_id)
        ON DELETE CASCADE,
    CONSTRAINT server_channels_kind_check CHECK (kind IN ('text'))
);

CREATE INDEX IF NOT EXISTS server_channels_server_created_idx
    ON server_channels (server_id, created_at ASC, channel_id ASC);

CREATE TABLE IF NOT EXISTS server_channel_messages (
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

CREATE INDEX IF NOT EXISTS server_channel_messages_channel_seq_idx
    ON server_channel_messages (channel_id, channel_seq DESC);

CREATE INDEX IF NOT EXISTS server_channel_messages_reply_idx
    ON server_channel_messages (reply_to_message_id);

CREATE TABLE IF NOT EXISTS server_channel_message_mentions (
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

CREATE INDEX IF NOT EXISTS server_channel_message_mentions_identity_idx
    ON server_channel_message_mentions (mentioned_identity_id, message_id);
