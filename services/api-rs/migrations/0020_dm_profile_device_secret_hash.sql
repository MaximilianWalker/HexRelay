ALTER TABLE dm_profile_devices
    ADD COLUMN IF NOT EXISTS device_secret_hash TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS dm_profile_devices_identity_device_secret_idx
    ON dm_profile_devices (identity_id, device_id, device_secret_hash);
