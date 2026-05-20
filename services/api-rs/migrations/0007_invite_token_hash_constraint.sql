DELETE FROM invites
WHERE token !~ '^[0-9a-f]{64}$';

ALTER TABLE invites
    ADD CONSTRAINT invites_token_sha256_hex
    CHECK (token ~ '^[0-9a-f]{64}$');
