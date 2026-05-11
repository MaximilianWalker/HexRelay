ALTER TABLE dm_policies
    DROP CONSTRAINT IF EXISTS dm_policies_offline_delivery_mode_check;

UPDATE dm_policies
SET offline_delivery_mode = 'encrypted_envelope_catchup'
WHERE offline_delivery_mode = 'best_effort_online';

ALTER TABLE dm_policies
    ADD CONSTRAINT dm_policies_offline_delivery_mode_check
    CHECK (offline_delivery_mode IN ('encrypted_envelope_catchup'));
