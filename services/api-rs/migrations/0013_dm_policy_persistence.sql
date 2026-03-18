CREATE TABLE IF NOT EXISTS dm_policies (
    identity_id TEXT PRIMARY KEY,
    inbound_policy TEXT NOT NULL,
    offline_delivery_mode TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT dm_policies_identity_fk
        FOREIGN KEY (identity_id)
        REFERENCES identity_keys(identity_id)
        ON DELETE CASCADE,
    CONSTRAINT dm_policies_inbound_policy_check
        CHECK (inbound_policy IN ('friends_only', 'same_server', 'anyone')),
    CONSTRAINT dm_policies_offline_delivery_mode_check
        CHECK (offline_delivery_mode IN ('best_effort_online'))
);
