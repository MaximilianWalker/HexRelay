CREATE TABLE IF NOT EXISTS dm_threads (
    thread_id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT dm_threads_kind_check CHECK (kind IN ('dm', 'group_dm'))
);

CREATE TABLE IF NOT EXISTS dm_thread_participants (
    thread_id TEXT NOT NULL,
    identity_id TEXT NOT NULL,
    last_read_seq BIGINT NOT NULL DEFAULT 0 CHECK (last_read_seq >= 0),
    PRIMARY KEY (thread_id, identity_id),
    CONSTRAINT dm_thread_participants_thread_fk
        FOREIGN KEY (thread_id)
        REFERENCES dm_threads(thread_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_thread_participants_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS dm_thread_participants_identity_idx
    ON dm_thread_participants (identity_id, thread_id);

CREATE TABLE IF NOT EXISTS dm_messages (
    message_id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    seq BIGINT NOT NULL CHECK (seq >= 0),
    ciphertext TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    edited_at TIMESTAMPTZ NULL,
    CONSTRAINT dm_messages_thread_fk
        FOREIGN KEY (thread_id)
        REFERENCES dm_threads(thread_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_messages_author_fk
        FOREIGN KEY (author_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_messages_thread_seq_unique UNIQUE (thread_id, seq)
);

CREATE INDEX IF NOT EXISTS dm_messages_thread_seq_idx
    ON dm_messages (thread_id, seq DESC);
