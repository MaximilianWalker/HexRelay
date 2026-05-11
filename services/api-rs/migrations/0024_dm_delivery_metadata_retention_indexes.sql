CREATE INDEX IF NOT EXISTS dm_fanout_delivery_log_created_idx
    ON dm_fanout_delivery_log (created_at);

CREATE INDEX IF NOT EXISTS dm_outbound_forwarding_log_retention_idx
    ON dm_outbound_forwarding_log (forwarding_state, created_at);
