# Migration Validation Evidence - 0026_dm_thread_last_message_summary

## Document Metadata

- Doc ID: migration-validation-0026-dm-thread-last-message-summary
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-17
- Source of truth: `evidence/migrations/0026_dm_thread_last_message_summary.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0026_dm_thread_last_message_summary.sql`.
- Primary edit location: update when DM thread summary columns or identity-scoped thread-list indexes change.
- Latest meaningful change: 2026-05-17 added per-thread last-message summary columns and the identity-scoped DM thread listing index.

## Migration Metadata

- Migration ID: `0026_dm_thread_last_message_summary`
- Owner: Maintainers
- Date (UTC): 2026-05-17
- Environment tested: local Rust validation; local Docker/Postgres unavailable, so Postgres-backed migration application is expected from CI `rust-check`
- Commit SHA: current branch commit containing this evidence
- PR number (or CI run ID): local validation before PR
- Generated at (UTC): `2026-05-17T23:23:58Z`

## Forward Validation

- Command(s) executed: temporary source-shape harness against `services/api-rs/src/infra/db/repos/dm_history_repo.rs`; `cargo test -p api-rs list_dm_threads_query_uses_identity_keyset_without_global_rank --all-features`; `cargo test -p api-rs --all-features`.
- Expected outcome: thread listing no longer uses global DM message/thread participant aggregates or `ROW_NUMBER()` ranking before `LIMIT`, and API tests compile/run with the new schema-facing code.
- Actual outcome: pass locally. The temporary harness failed before the fix on the cited global aggregate/rank markers and passed after the fix; focused durable regression and API tests passed.
- Evidence path (logs/artifacts): local CLI validation and this file.

## Idempotency and Re-run Check

- Re-run command(s): migration review plus `cargo test -p api-rs --all-features`.
- Expected outcome: additive columns and index are guarded with `IF NOT EXISTS`; named check constraints are added only when absent; existing DM thread summaries are backfilled from current `dm_messages`.
- Actual outcome: pass for local code/test validation. Direct Postgres re-run validation was not available because Docker was not reachable and `127.0.0.1:5432` was closed in this run.
- Evidence path: `services/api-rs/migrations/0026_dm_thread_last_message_summary.sql`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore a database backup taken before migration, or drop `dm_thread_participants_identity_last_message_idx`, `dm_thread_participants.last_message_seq`, and `dm_threads.last_message_seq` / `last_message_preview` / `last_message_at` during controlled rollback.
- Expected outcome: the previous aggregate query shape can be restored with code rollback; no DM message ciphertext, participant membership, or read cursor rows are rewritten by the forward migration.
- Actual outcome: additive schema plus summary backfill only; canonical DM message rows remain unchanged.
- Evidence path: repository migration and focused API validation.

## Data Integrity Verification

- Constraints/indexes verified: `dm_threads_last_message_seq_nonnegative`, `dm_thread_participants_last_message_seq_nonnegative`, and `dm_thread_participants_identity_last_message_idx`.
- Row-count or key invariants checked: migration updates summary columns from existing `dm_messages` and does not insert or delete `dm_threads`, `dm_thread_participants`, or `dm_messages` rows.
- Evidence path: `services/api-rs/migrations/0026_dm_thread_last_message_summary.sql`; `services/api-rs/src/infra/db/repos/dm_history_repo.rs`.

## Sign-off

- Reviewer: Codex agent
- Decision: pass
- Notes: Local Postgres was unavailable in this run; CI should exercise migration application through the API service database preparation path.
