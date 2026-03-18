# Migration Validation Evidence - 0016_dm_history_local_store

## Document Metadata

- Doc ID: migration-validation-0016-dm-history-local-store
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-18
- Source of truth: `evidence/migrations/0016_dm_history_local_store.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0016_dm_history_local_store.sql`.
- Primary edit location: update when local DM history persistence evidence changes.
- Latest meaningful change: 2026-03-18 added delivery-pass evidence for local-runtime-backed DM thread/message persistence replacing fixture-backed history.

## Migration Metadata

- Migration ID: `0016_dm_history_local_store`
- Owner: Maintainers
- Date (UTC): 2026-03-18
- Environment tested: local dev (Windows) + CI workflow (`migration-evidence-check`, `rust-check`, `rust-coverage-gate`)
- Commit SHA: `1e16e6d`
- PR number (or CI run ID): `PR #42` / `23242910720`
- Generated at (UTC): `2026-03-18T12:05:00Z`

## Forward Validation

- Command(s) executed: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`
- Expected outcome: DM thread and message listing use persisted local-runtime storage with participant scoping instead of fixture-only responses.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation for `services/api-rs` in the PR #42 delivery session.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test --all-features`
- Expected outcome: repeated startup and seed/update flows upsert thread, participant, and message rows without duplicate-key failures, while member scoping remains intact.
- Actual outcome: pass.
- Evidence path: local CLI output from repeated API test execution in the delivery session.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive schema in application-level validation pass.
- Expected outcome: recovery uses documented restore procedures if rollback is required.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md`

## Data Integrity Verification

- Constraints/indexes verified: `dm_threads` primary key and kind check, `dm_thread_participants` composite primary key plus thread/identity FKs, `dm_thread_participants_identity_idx`, `dm_messages` primary key plus thread/author FKs and `(thread_id, seq)` uniqueness, `dm_messages_thread_seq_idx`.
- Row-count or key invariants checked: authenticated thread/message listing remains participant-scoped and non-members receive `thread_not_found`; history persists as local-runtime-backed state rather than server-authoritative DM history.
- Evidence path: `services/api-rs/src/tests/integration/dm_threads_tests.rs`, `services/api-rs/src/infra/db/repos/dm_history_repo.rs`, `services/api-rs/src/transport/http/handlers/dm.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration persists local/user-owned DM history surfaces only and stays separate from the unresolved replay-backlog durability watch.
