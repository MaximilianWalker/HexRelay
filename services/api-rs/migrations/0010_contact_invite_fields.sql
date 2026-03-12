ALTER TABLE invites
    ADD COLUMN IF NOT EXISTS invite_id TEXT;

ALTER TABLE invites
    ADD COLUMN IF NOT EXISTS creator_identity_id TEXT;

CREATE INDEX IF NOT EXISTS invites_creator_identity_created_idx
    ON invites (creator_identity_id, created_at DESC);
