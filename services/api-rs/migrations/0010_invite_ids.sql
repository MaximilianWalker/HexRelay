ALTER TABLE invites
    ADD COLUMN IF NOT EXISTS invite_id TEXT;

DELETE FROM invites
WHERE invite_id IS NULL OR BTRIM(invite_id) = '';

ALTER TABLE invites
    ALTER COLUMN invite_id SET NOT NULL;

ALTER TABLE invites
    ADD CONSTRAINT invites_invite_id_unique UNIQUE (invite_id);

DROP INDEX IF EXISTS invites_creator_identity_created_idx;

ALTER TABLE invites
    DROP COLUMN IF EXISTS creator_identity_id;
