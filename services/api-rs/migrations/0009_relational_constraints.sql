DELETE FROM sessions
WHERE identity_id NOT IN (SELECT identity_id FROM identity_keys);

DELETE FROM auth_challenges
WHERE identity_id NOT IN (SELECT identity_id FROM identity_keys);

DELETE FROM friend_requests
WHERE requester_identity_id NOT IN (SELECT identity_id FROM identity_keys)
   OR target_identity_id NOT IN (SELECT identity_id FROM identity_keys);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'sessions_identity_fk'
    ) THEN
        ALTER TABLE sessions
            ADD CONSTRAINT sessions_identity_fk
            FOREIGN KEY (identity_id)
            REFERENCES identity_keys(identity_id)
            ON DELETE CASCADE;
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'auth_challenges_identity_fk'
    ) THEN
        ALTER TABLE auth_challenges
            ADD CONSTRAINT auth_challenges_identity_fk
            FOREIGN KEY (identity_id)
            REFERENCES identity_keys(identity_id)
            ON DELETE CASCADE;
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'friend_requests_requester_fk'
    ) THEN
        ALTER TABLE friend_requests
            ADD CONSTRAINT friend_requests_requester_fk
            FOREIGN KEY (requester_identity_id)
            REFERENCES identity_keys(identity_id)
            ON DELETE CASCADE;
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'friend_requests_target_fk'
    ) THEN
        ALTER TABLE friend_requests
            ADD CONSTRAINT friend_requests_target_fk
            FOREIGN KEY (target_identity_id)
            REFERENCES identity_keys(identity_id)
            ON DELETE CASCADE;
    END IF;
END $$;
