# Migration Validation Evidence - 0020_dm_profile_device_secret_hash

## Document Metadata

- Doc ID: migration-validation-0020-dm-profile-device-secret-hash
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `evidence/migrations/0020_dm_profile_device_secret_hash.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0020_dm_profile_device_secret_hash.sql`.
- Primary edit location: update when validation evidence for DM profile-device secret binding changes.
- Latest meaningful change: 2026-05-11 added profile-device secret hash storage for DM catch-up and ack binding.

## Migration Metadata

- Migration ID: `0020_dm_profile_device_secret_hash`
- Owner: Maintainers
- Date (UTC): 2026-05-11
- Environment tested: local dev (Windows) with targeted API/realtime/web checks and contract guardrails
- Commit SHA: pending current branch
- PR number (or CI run ID): pending current PR
- Generated at (UTC): `2026-05-11T00:00:00Z`

## Forward Validation

- Command(s) executed: `cargo check -p api-rs`, `cargo check -p realtime-rs`, `cargo test -p api-rs fanout_ack`, `cargo test -p api-rs dm_fanout`, `cargo test -p api-rs dm_envelope_dispatch_ack_persists_through_realtime_websocket`, `cargo test -p realtime-rs dm`, `npm --prefix apps/web test -- lib/api.test.ts lib/dm-realtime.test.ts`, `bash scripts/validators/contract-parity.sh HEAD HEAD`, `bash scripts/validators/dm-transport-policy.sh`.
- Expected outcome: `dm_profile_devices` gains a non-plaintext `device_secret_hash` column and runtime DM catch-up/ack paths reject mismatched device secrets without storing raw device secrets.
- Actual outcome: pending final validation in current hardening pass.
- Evidence path (logs/artifacts): local CLI validation in the DM E2EE envelope baseline pivot branch.

## Idempotency and Re-run Check

- Re-run command(s): `cargo check -p api-rs`, `cargo test -p api-rs fanout_ack`, `cargo test -p api-rs dm_fanout`.
- Expected outcome: `ADD COLUMN IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` remain safe when re-run, and DM profile-device heartbeat/catch-up/ack tests pass with existing rows backfilled to an empty hash until first valid heartbeat.
- Actual outcome: pending final validation in current hardening pass.
- Evidence path: `services/api-rs/migrations/0020_dm_profile_device_secret_hash.sql`, `services/api-rs/src/infra/db/repos/dm_repo.rs`, `services/api-rs/src/transport/http/handlers/dm.rs`, `services/api-rs/src/tests/integration/dm_fanout_catch_up_tests.rs`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore database backup before migration if device-secret hash state must be removed.
- Expected outcome: operators can restore pre-migration profile-device rows; runtime requiring device-secret binding should not run against a rolled-back schema.
- Actual outcome: acknowledged.
- Evidence path: `docs/operations/01-mvp-runbook.md`, `docs/planning/infra-free-dm-connectivity-execution-plan.md`.

## Data Integrity Verification

- Constraints/indexes verified: profile-device primary key remains `(identity_id, device_id)`; new `device_secret_hash` is non-null and indexed with identity/device for ack/catch-up binding checks.
- Row-count or key invariants checked: existing profile-device rows remain addressable; mismatched device-secret updates are rejected rather than rebinding an existing device id.
- Evidence path: `services/api-rs/src/infra/db/repos/dm_repo.rs`, `services/api-rs/src/tests/integration/dm_fanout_catch_up_tests.rs`.

## Sign-off

- Reviewer: OpenCode agent (hardening validation pass)
- Decision: pending final validation
- Notes: The migration stores only a hash of the per-device secret; raw device secrets remain client/realtime-connection inputs and are not echoed in public realtime ack events.
