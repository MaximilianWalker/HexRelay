# Migration Validation Evidence - 0019_remove_dm_direct_connect_tables

## Document Metadata

- Doc ID: migration-validation-0019-remove-dm-direct-connect-tables
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `evidence/migrations/0019_remove_dm_direct_connect_tables.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0019_remove_dm_direct_connect_tables.sql`.
- Primary edit location: update when validation evidence for retired DM direct-connect tables changes.
- Latest meaningful change: 2026-05-11 aligned validation evidence wording with the server-node P2P E2EE envelope pivot.

## Migration Metadata

- Migration ID: `0019_remove_dm_direct_connect_tables`
- Owner: Maintainers
- Date (UTC): 2026-05-08
- Environment tested: local dev (Windows) with Rust checks, targeted API tests, web checks, and guardrail/parity checks
- Commit SHA: pending current branch
- PR number (or CI run ID): pending current PR
- Generated at (UTC): `2026-05-08T00:00:00Z`

## Forward Validation

- Command(s) executed: `cargo check -p api-rs`, `cargo test -p api-rs dm_fanout`, `cargo test -p api-rs friends`, `bash scripts/validate-dm-transport-policy.sh`, `bash scripts/validate-contract-parity.sh origin/master HEAD`
- Expected outcome: DB startup can include migration `0019_remove_dm_direct_connect_tables` after earlier node-bypassing migrations, while runtime API and contract surfaces no longer reference pairing nonce or endpoint-card tables.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation in the DM E2EE envelope baseline pivot branch.

## Idempotency and Re-run Check

- Re-run command(s): `cargo check -p api-rs`, `cargo test -p api-rs dm_fanout`, `cargo test -p api-rs friends`
- Expected outcome: `DROP TABLE IF EXISTS` remains safe when obsolete tables are already absent, and DB-backed DM fanout/friend bootstrap paths continue to compile and pass without direct-connect table dependencies.
- Actual outcome: pass.
- Evidence path: `services/api-rs/migrations/0019_remove_dm_direct_connect_tables.sql`, `services/api-rs/src/db.rs`, `services/api-rs/src/transport/http/handlers/dm.rs`, `services/api-rs/src/transport/http/handlers/friends.rs`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for intentional destructive cleanup of retired node-bypassing DM tables; recovery uses database backup/restore before migration application if obsolete data must be inspected.
- Expected outcome: operators recover removed historical direct-connect table data from backups rather than reintroducing runtime table dependencies.
- Actual outcome: acknowledged; retired data is intentionally not used by current runtime.
- Evidence path: `docs/operations/01-mvp-runbook.md`, `docs/product/03-clarifications.md`, `docs/planning/infra-free-dm-connectivity-execution-plan.md`.

## Data Integrity Verification

- Constraints/indexes verified: obsolete `dm_pairing_nonces` and `dm_endpoint_cards` tables are dropped only if present; retained `dm_profile_devices`, DM policy, DM history, and durable delivery-log tables remain unchanged.
- Row-count or key invariants checked: DM fanout, catch-up, and accepted-friend identity bootstrap tests pass without endpoint-card or pairing-nonce table access.
- Evidence path: `services/api-rs/src/tests/integration/dm_fanout_tests.rs`, `services/api-rs/src/tests/integration/dm_fanout_catch_up_tests.rs`, `services/api-rs/src/tests/integration/friends_tests.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration deliberately removes persisted node-bypassing DM bootstrap/control leftovers after runtime and contract surfaces moved to server-node P2P E2EE envelope delivery.
