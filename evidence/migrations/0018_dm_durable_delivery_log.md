# Migration Validation Evidence - 0018_dm_durable_delivery_log

## Document Metadata

- Doc ID: migration-validation-0018-dm-durable-delivery-log
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-09
- Source of truth: `evidence/migrations/0018_dm_durable_delivery_log.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0018_dm_durable_delivery_log.sql`.
- Primary edit location: update when validation evidence for durable DM acceptance metadata changes.
- Latest meaningful change: 2026-04-09 added validation evidence for persisted DM delivery-log state, canonical message acceptance, and restart-proof catch-up behavior.

## Migration Metadata

- Migration ID: `0018_dm_durable_delivery_log`
- Owner: Maintainers
- Date (UTC): 2026-04-09
- Environment tested: local dev (Windows) + GitHub Actions (`migration-evidence-check`, Rust checks, parity/docs checks)
- Commit SHA: `c9ef176`
- PR number (or CI run ID): `PR #88` / pending current CI run
- Generated at (UTC): `2026-04-09T00:00:00Z`

## Forward Validation

- Command(s) executed: `cargo test -p api-rs fanout_cursor_metadata_persists_across_db_restart`, `cargo test -p api-rs accepted_dm_without_active_devices_survives_restart_and_catches_up_later`, `cargo test -p api-rs fanout_dispatch_delivers_to_all_active_profile_devices`, `cargo check -p api-rs`, `python -c "from scripts.contract_parity import engine; raise SystemExit(engine.validate_api_semantic_contracts('docs/contracts/runtime-rest-v1.openapi.yaml'))"`
- Expected outcome: accepted DM send persists canonical DM history plus durable delivery metadata before success, and catch-up survives restart when DB storage is configured.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation in the PR #88 delivery session.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test -p api-rs accepted_dm_without_active_devices_survives_restart_and_catches_up_later`, `cargo test -p api-rs fanout_cursor_metadata_persists_across_db_restart`
- Expected outcome: repeated DB startup and replay/cursor flows remain deterministic without duplicate-key or replay-regression failures.
- Actual outcome: pass.
- Evidence path: local CLI rerun output captured during PR #88 delivery.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive schema in application-level validation pass.
- Expected outcome: recovery uses documented database restore procedures instead of destructive in-place rollback.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md`

## Data Integrity Verification

- Constraints/indexes verified: primary key `(identity_id, cursor)`, foreign keys to `identity_keys`, `dm_threads`, and `dm_messages`, plus `dm_fanout_delivery_log_identity_cursor_idx`.
- Row-count or key invariants checked: accepted DM fanout creates canonical thread/message history before success, persists delivery metadata for the recipient identity, and allows later catch-up after restart.
- Evidence path: `services/api-rs/src/transport/http/handlers/dm.rs`, `services/api-rs/src/infra/db/repos/dm_repo.rs`, `services/api-rs/src/infra/db/repos/dm_history_repo.rs`, `services/api-rs/src/tests/integration/dm_policy_tests.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration intentionally makes durable sender-side acceptance and persisted delivery metadata explicit for DB-backed DM reliability before broader delivery-state and reachability follow-up work.
