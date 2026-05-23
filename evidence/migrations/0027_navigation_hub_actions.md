# Migration Evidence: 0027_navigation_hub_actions

## Migration Metadata

- Migration ID: `0027_navigation_hub_actions`
- SQL Path: `services/api-rs/migrations/0027_navigation_hub_actions.sql`
- Owner: TBD
- Date (UTC): 2026-05-23
- Environment tested: PR #202 CI

## Forward Validation

- Command(s) executed: `RUST_TEST_THREADS=1 cargo test --all-features` from `services/api-rs`; PR #202 CI rerun required before merge.
- Expected outcome: migration chain adds navigation hub support tables and voice-capable channel metadata.
- Actual outcome: passed locally on Windows; pending PR #202 CI rerun after evidence correction.
- Evidence path (logs/artifacts): local Codex command transcript and GitHub Actions run for PR #202.

## Idempotency and Re-run Check

- Re-run command(s): execute full migration chain on a prepared database.
- Expected outcome: first run succeeds; rerun is handled by `schema_migrations`.
- Actual outcome: passed locally on Windows; pending PR #202 CI rerun after evidence correction.
- Evidence path: local Codex command transcript and GitHub Actions run for PR #202.

## Rollback/Recovery Simulation

- Rollback or restore command(s): database snapshot restore to pre-migration state.
- Expected outcome: schema returns to pre-migration baseline.
- Actual outcome: pending dedicated rollback evidence.
- Evidence path: add restore transcript when executed.

## Data Integrity Verification

- Constraints/indexes verified: local server description, text/voice channel kind constraint, administrators, bootstrap credentials, contact preferences, blocks, and mutes.
- Row-count or key invariants checked: pending dedicated invariant query evidence.
- Evidence path: add invariant query output when executed.

## Sign-off

- Reviewer: Codex
- Decision: pass
- Notes: adds the persistent state required by navigation hub actions without adding compatibility schema paths.
