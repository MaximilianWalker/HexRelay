ALTER TABLE dm_threads
    ADD COLUMN IF NOT EXISTS last_message_seq BIGINT NOT NULL DEFAULT 0;

ALTER TABLE dm_threads
    ADD COLUMN IF NOT EXISTS last_message_preview TEXT NOT NULL DEFAULT '';

ALTER TABLE dm_threads
    ADD COLUMN IF NOT EXISTS last_message_at TIMESTAMPTZ NULL;

ALTER TABLE dm_thread_participants
    ADD COLUMN IF NOT EXISTS last_message_seq BIGINT NOT NULL DEFAULT 0;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'dm_threads_last_message_seq_nonnegative'
    ) THEN
        ALTER TABLE dm_threads
            ADD CONSTRAINT dm_threads_last_message_seq_nonnegative CHECK (last_message_seq >= 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'dm_thread_participants_last_message_seq_nonnegative'
    ) THEN
        ALTER TABLE dm_thread_participants
            ADD CONSTRAINT dm_thread_participants_last_message_seq_nonnegative CHECK (last_message_seq >= 0);
    END IF;
END $$;

WITH latest AS (
    SELECT DISTINCT ON (thread_id)
        thread_id,
        seq,
        ciphertext,
        created_at
    FROM dm_messages
    ORDER BY thread_id, seq DESC
)
UPDATE dm_threads t
SET last_message_seq = latest.seq,
    last_message_preview = latest.ciphertext,
    last_message_at = latest.created_at
FROM latest
WHERE t.thread_id = latest.thread_id;

UPDATE dm_thread_participants pt
SET last_message_seq = t.last_message_seq
FROM dm_threads t
WHERE pt.thread_id = t.thread_id;

CREATE INDEX IF NOT EXISTS dm_thread_participants_identity_last_message_idx
    ON dm_thread_participants (identity_id, last_message_seq DESC, thread_id ASC);
