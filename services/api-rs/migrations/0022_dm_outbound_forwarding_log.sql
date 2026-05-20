CREATE TABLE IF NOT EXISTS dm_outbound_forwarding_log (
    sender_identity_id TEXT NOT NULL,
    destination_server_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    thread_id TEXT NOT NULL,
    recipient_identity_id TEXT NOT NULL,
    ciphertext TEXT NOT NULL,
    source_device_id TEXT NULL,
    delivery_cursor BIGINT NOT NULL CHECK (delivery_cursor > 0),
    forwarding_state TEXT NOT NULL,
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    last_error TEXT NULL,
    last_attempt_at TIMESTAMPTZ NULL,
    forwarded_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (sender_identity_id, destination_server_id, message_id),
    CONSTRAINT dm_outbound_forwarding_log_sender_fk
        FOREIGN KEY (sender_identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_outbound_forwarding_log_state_check
        CHECK (forwarding_state IN ('queued', 'forwarded', 'failed'))
);

CREATE INDEX IF NOT EXISTS dm_outbound_forwarding_log_sender_created_idx
    ON dm_outbound_forwarding_log (sender_identity_id, created_at DESC);

CREATE INDEX IF NOT EXISTS dm_outbound_forwarding_log_destination_state_idx
    ON dm_outbound_forwarding_log (destination_server_id, forwarding_state);
