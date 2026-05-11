# Migration Validation Evidence - 0024_dm_delivery_metadata_retention_indexes

## Document Metadata

- Doc ID: migration-validation-0024-dm-delivery-metadata-retention-indexes
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `evidence/migrations/0024_dm_delivery_metadata_retention_indexes.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0024_dm_delivery_metadata_retention_indexes.sql`.
- Primary edit location: update when DM delivery metadata retention indexes or purge predicates change.
- Latest meaningful change: 2026-05-11 added retention indexes for DM fanout delivery metadata and outbound forwarding metadata cleanup.

## Migration Metadata

- Migration ID: `0024_dm_delivery_metadata_retention_indexes`
- Owner: Maintainers
- Date (UTC): 2026-05-11
- Environment tested: local dev with API retention and repeat-run test coverage
- Commit SHA: current branch commit containing this evidence
- PR number (or CI run ID): local validation before PR
- Generated at (UTC): `2026-05-11T19:46:25Z`

## Forward Validation

- Command(s) executed: `cargo test -p api-rs dm_delivery_metadata_retention_purges_only_expired_delivery_metadata`; `cargo test -p api-rs fanout_dispatch_rate_limits_per_sender_identity`; `cargo test -p api-rs parses_dm_retention_and_rate_limit_config`; `cargo test -p api-rs`; `cargo clippy -p api-rs --all-targets -- -D warnings`.
- Expected outcome: retention purge predicates can use indexed `created_at` scans for `dm_fanout_delivery_log` and state/age scans for `dm_outbound_forwarding_log`.
- Actual outcome: pass. API tests confirmed expired fanout/outbound forwarding metadata deletion, canonical `dm_messages` ciphertext retention, queued outbound forwarding preservation, sender-scoped dispatch rate limiting, and env-config parsing.
- Evidence path (logs/artifacts): local CLI validation and this file.

## Idempotency and Re-run Check

- Re-run command(s): migration application through `connect_and_prepare`.
- Expected outcome: indexes are created with `IF NOT EXISTS` and repeated migration application is guarded by `schema_migrations`.
- Actual outcome: pass through `cargo test -p api-rs`; the API integration suite exercises `connect_and_prepare` against the local dev database and all schema migrations completed.
- Evidence path: `services/api-rs/migrations/0024_dm_delivery_metadata_retention_indexes.sql`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore a database backup taken before migration, or drop the two retention indexes during controlled rollback.
- Expected outcome: runtime correctness does not depend on the indexes, but retention purge performance may degrade without them on large delivery logs.
- Actual outcome: additive indexes only; no stored data is rewritten.
- Evidence path: repository migration and retention tests.

## Data Integrity Verification

- Constraints/indexes verified: `dm_fanout_delivery_log_created_idx` and `dm_outbound_forwarding_log_retention_idx`.
- Row-count or key invariants checked: migration adds indexes only and does not alter DM message, fanout delivery, cursor, or outbound forwarding rows.
- Evidence path: `services/api-rs/migrations/0024_dm_delivery_metadata_retention_indexes.sql`.

## Sign-off

- Reviewer: Codex agent
- Decision: pass
- Notes: This migration supports server-node/message-node encrypted-envelope metadata retention and does not add direct user-to-user transport or UX behavior.
