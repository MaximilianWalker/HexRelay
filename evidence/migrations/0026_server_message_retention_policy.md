# Migration Validation Evidence - 0026_server_message_retention_policy

## Document Metadata

- Doc ID: migration-validation-0026-server-message-retention-policy
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-19
- Source of truth: `evidence/migrations/0026_server_message_retention_policy.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0026_server_message_retention_policy.sql`.
- Primary edit location: update when server message retention policy storage or retention lookup indexes change.
- Latest meaningful change: 2026-05-19 added per-server channel-message retention policy storage and tombstone-enforcement evidence.

## Migration Metadata

- Migration ID: `0026_server_message_retention_policy`
- Owner: Maintainers
- Date (UTC): 2026-05-19
- Environment tested: local dev with Postgres-backed API integration tests when database is available; non-DB static validation otherwise
- Commit SHA: current branch commit containing this evidence
- PR number (or CI run ID): local validation before PR
- Generated at (UTC): `2026-05-19T12:30:59Z`

## Forward Validation

- Command(s) executed: `cargo test -p api-rs server_message_retention_tombstones_expired_channel_history --all-features`; `cargo test -p api-rs --all-features`.
- Expected outcome: migration applies through `connect_and_prepare`, `servers.retention_message_days` exists with a null-or-positive-days constraint, expired server-channel messages are tombstoned, and unexpired messages remain readable.
- Actual outcome: pass. The focused Postgres-backed API integration test applied the migration through `connect_and_prepare`, set `servers.retention_message_days = 1`, listed channel history, and verified expired message content/mentions were tombstoned while an unexpired message remained unchanged. The full serialized API suite also passed.
- Evidence path (logs/artifacts): local CLI validation and this file.

## Idempotency and Re-run Check

- Re-run command(s): migration application through `connect_and_prepare` in the focused API integration test and the API test suite.
- Expected outcome: migration is recorded in `schema_migrations`; additive column and index creation are idempotent; re-running does not rewrite existing server/channel/message data.
- Actual outcome: pass through focused API integration coverage and `RUST_TEST_THREADS=1 cargo test -p api-rs --all-features`.
- Evidence path: `services/api-rs/migrations/0026_server_message_retention_policy.sql`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore a database backup taken before migration, or drop `server_channel_messages_retention_idx`, `servers_retention_message_days_check`, and `servers.retention_message_days` during a controlled rollback.
- Expected outcome: existing server/channel/message rows are not rewritten by the forward migration; retention enforcement activates only for servers with a non-null policy value.
- Actual outcome: additive schema only; no stored message rows are rewritten by migration application.
- Evidence path: repository migration and focused server-channel message retention tests.

## Data Integrity Verification

- Constraints/indexes verified: `servers_retention_message_days_check` and `server_channel_messages_retention_idx`.
- Row-count or key invariants checked: retention tombstoning keeps expired message ids and channel sequence positions listable while scrubbing content and mentions; unexpired messages remain unchanged.
- Evidence path: `services/api-rs/src/tests/integration/server_channel_messages_tests.rs`.

## Sign-off

- Reviewer: Codex agent
- Decision: pass
- Notes: This migration adds backend/API policy storage and does not add product UI or UX behavior.
