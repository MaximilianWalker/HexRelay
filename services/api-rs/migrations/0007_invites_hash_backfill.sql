CREATE EXTENSION IF NOT EXISTS pgcrypto;

UPDATE invites
SET token = encode(digest(token, 'sha256'), 'hex')
WHERE token !~ '^[0-9a-f]{64}$';
