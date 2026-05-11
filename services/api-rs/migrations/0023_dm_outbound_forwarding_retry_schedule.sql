ALTER TABLE dm_outbound_forwarding_log
    ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ NULL;

UPDATE dm_outbound_forwarding_log
SET next_attempt_at = NOW(),
    updated_at = NOW()
WHERE forwarding_state IN ('queued', 'failed')
  AND next_attempt_at IS NULL;

CREATE INDEX IF NOT EXISTS dm_outbound_forwarding_log_retry_idx
    ON dm_outbound_forwarding_log (forwarding_state, next_attempt_at, attempt_count);
