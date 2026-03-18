# Migration Validation Evidence - 0015_dm_fanout_cursor_metadata

## Document Metadata

- Doc ID: migration-validation-0015-dm-fanout-cursor-metadata
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-18
- Source of truth: `evidence/migrations/0015_dm_fanout_cursor_metadata.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0015_dm_fanout_cursor_metadata.sql`.
- Primary edit location: update when persisted DM fanout cursor evidence changes.
- Latest meaningful change: 2026-03-18 added delivery-pass evidence for restart-stable fanout stream-head and per-device cursor metadata.

## Migration Metadata

- Migration ID: `0015_dm_fanout_cursor_metadata`
- Owner: Maintainers
- Date (UTC): 2026-03-18
- Environment tested: local dev (Windows) + CI workflow (`migration-evidence-check`, `rust-check`, `rust-coverage-gate`)
- Commit SHA: `1e16e6d`
- PR number (or CI run ID): `PR #42` / `23242910720`
- Generated at (UTC): `2026-03-18T12:05:00Z`

## Forward Validation

- Command(s) executed: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`
- Expected outcome: DM fanout stream-head and per-device cursor checkpoints persist across restart and continue to gate catch-up/pruning behavior.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation for `services/api-rs` in the PR #42 delivery session.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test --all-features`
- Expected outcome: repeated startup and fanout catch-up flows update existing cursor rows deterministically without duplicate metadata records.
- Actual outcome: pass.
- Evidence path: local CLI output from repeated API test execution in the delivery session.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive schema in application-level validation pass.
- Expected outcome: recovery uses documented restore procedures if rollback is required.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md`

## Data Integrity Verification

- Constraints/indexes verified: `dm_fanout_stream_heads` primary key and identity FK, `dm_fanout_device_cursors` composite primary key and device FK, `dm_fanout_device_cursors_identity_idx`.
- Row-count or key invariants checked: per-device cursor checkpoints remain monotonic and fanout replay state still excludes durable encrypted payload backlog storage.
- Evidence path: `services/api-rs/src/tests/integration/dm_policy_tests.rs`, `services/api-rs/src/transport/http/handlers/dm.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration persists control metadata only; encrypted replay payload backlog remains intentionally bounded and non-durable.
