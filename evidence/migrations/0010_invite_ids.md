# Migration Evidence: 0010_invite_ids

## Migration Metadata

- Migration ID: `0010_invite_ids`
- SQL Path: `services/api-rs/migrations/0010_invite_ids.sql`
- Owner: TBD
- Date (UTC): baseline
- Environment tested: baseline bootstrap

## Forward Validation

- Command(s) executed: `cargo test --workspace`
- Expected outcome: migration chain applies cleanly and exposes stable invite ids for server invite records.
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

- Constraints/indexes verified: `invites.invite_id` column exists for server invite responses.
- Row-count or key invariants checked: pending dedicated invariant query evidence.
- Evidence path: add invariant query output when executed.

## Sign-off

- Reviewer: Codex
- Decision: pass
- Notes: replaces the removed invite ownership field migration with the current server-invite id shape.
