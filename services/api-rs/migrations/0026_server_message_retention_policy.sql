ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS retention_message_days INTEGER NULL;

ALTER TABLE servers
    DROP CONSTRAINT IF EXISTS servers_retention_message_days_check;

ALTER TABLE servers
    ADD CONSTRAINT servers_retention_message_days_check
        CHECK (retention_message_days IS NULL OR retention_message_days >= 1);

CREATE INDEX IF NOT EXISTS server_channel_messages_retention_idx
    ON server_channel_messages (channel_id, created_at)
    WHERE deleted_at IS NULL;
