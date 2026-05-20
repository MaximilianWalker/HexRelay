# Migration Evidence: 0007_invite_token_hash_constraint

## Migration Metadata

- Migration ID: `0007_invite_token_hash_constraint`
- SQL Path: `services/api-rs/migrations/0007_invite_token_hash_constraint.sql`
- Owner: TBD
- Date (UTC): baseline
- Environment tested: baseline bootstrap

## Forward Validation

- Command(s) executed: `cargo test --workspace`
- Expected outcome: migration chain applies cleanly for test databases.
- Actual outcome: passed.
- Evidence path (logs/artifacts): command transcript in Codex run.

## Idempotency and Re-run Check

- Re-run command(s): execute full migration chain on a prepared database.
- Expected outcome: first run succeeds; rerun is handled by `schema_migrations`.
- Actual outcome: passed through migration tracking in the workspace test database.
- Evidence path: command transcript in Codex run.

## Rollback/Recovery Simulation

- Rollback or restore command(s): database snapshot restore to pre-migration state.
- Expected outcome: schema returns to pre-migration baseline.
- Actual outcome: pending dedicated rollback evidence.
- Evidence path: add restore transcript when executed.

## Data Integrity Verification

- Constraints/indexes verified: invite tokens must match SHA-256 hex storage format.
- Row-count or key invariants checked: pending dedicated invariant query evidence.
- Evidence path: add invariant query output when executed.

## Sign-off

- Reviewer: Codex
- Decision: pass
- Notes: destructive MVP migration path deletes plaintext invite-token rows and enforces hashed-token storage instead of carrying compatibility backfill behavior.
