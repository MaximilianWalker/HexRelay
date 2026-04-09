CREATE TABLE IF NOT EXISTS dm_fanout_delivery_log (
    identity_id TEXT NOT NULL,
    cursor BIGINT NOT NULL CHECK (cursor >= 0),
    thread_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    sender_identity_id TEXT NOT NULL,
    ciphertext TEXT NOT NULL,
    source_device_id TEXT NULL,
    delivery_state TEXT NOT NULL,
    reachability_state TEXT NOT NULL,
    delivered_device_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (identity_id, cursor),
    CONSTRAINT dm_fanout_delivery_log_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_fanout_delivery_log_thread_fk
        FOREIGN KEY (thread_id)
        REFERENCES dm_threads(thread_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_fanout_delivery_log_message_fk
        FOREIGN KEY (message_id)
        REFERENCES dm_messages(message_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_fanout_delivery_log_sender_fk
        FOREIGN KEY (sender_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS dm_fanout_delivery_log_identity_cursor_idx
    ON dm_fanout_delivery_log (identity_id, cursor DESC);
