# Migration Validation Evidence - 0021_dm_encrypted_envelope_catchup_policy

## Document Metadata

- Doc ID: migration-validation-0021-dm-encrypted-envelope-catchup-policy
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `evidence/migrations/0021_dm_encrypted_envelope_catchup_policy.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0021_dm_encrypted_envelope_catchup_policy.sql`.
- Primary edit location: update when validation evidence for DM offline delivery policy mode changes.
- Latest meaningful change: 2026-05-11 replaced the stale best-effort online policy mode with encrypted-envelope catch-up.

## Migration Metadata

- Migration ID: `0021_dm_encrypted_envelope_catchup_policy`
- Owner: Maintainers
- Date (UTC): 2026-05-11
- Environment tested: local dev (Windows) with targeted API/web checks and DM guardrail
- Commit SHA: pending current branch
- PR number (or CI run ID): pending current PR
- Generated at (UTC): `2026-05-11T00:00:00Z`

## Forward Validation

- Command(s) executed: `cargo test -p api-rs dm_policy`, `cargo test -p api-rs validates_fanout`, `cargo test -p api-rs dev_seed`, `npm --prefix apps/web test -- lib/api.test.ts`, `cargo test -p communication-core`, `node scripts/validators/dm-transport-policy.mjs`.
- Expected outcome: existing `dm_policies.offline_delivery_mode=best_effort_online` rows are rewritten to `encrypted_envelope_catchup`, and new rows are constrained to the current encrypted-envelope catch-up mode.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation in the current architecture-alignment pass.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test -p api-rs dm_policy`, `cargo test -p api-rs dev_seed`.
- Expected outcome: the migration can run once in normal migration order after `0019_remove_dm_direct_connect_tables`; subsequent application is prevented by `schema_migrations`, and the final constraint only accepts `encrypted_envelope_catchup`.
- Actual outcome: pass.
- Evidence path: `services/api-rs/migrations/0021_dm_encrypted_envelope_catchup_policy.sql`, `services/api-rs/src/domain/dm/validation.rs`, `services/api-rs/src/tests/integration/dm_policy_tests.rs`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore database backup before migration if the old policy mode must be inspected.
- Expected outcome: runtime using the current server encrypted-envelope delivery baseline should not run against a restored schema that only accepts `best_effort_online`.
- Actual outcome: acknowledged.
- Evidence path: `docs/product/01-mvp-plan.md`, `docs/product/02-prd.md`, `docs/contracts/runtime-rest.openapi.yaml`.

## Data Integrity Verification

- Constraints/indexes verified: `dm_policies_offline_delivery_mode_check` is replaced with an `encrypted_envelope_catchup`-only check.
- Row-count or key invariants checked: policy rows keep their identity and inbound policy values while only `offline_delivery_mode` is rewritten.
- Evidence path: `services/api-rs/migrations/0021_dm_encrypted_envelope_catchup_policy.sql`.

## Sign-off

- Reviewer: Codex agent (architecture-alignment validation pass)
- Decision: pass
- Notes: This migration aligns the persisted policy mode with the already documented durable encrypted-envelope acceptance plus catch-up semantics; it does not add a new delivery feature.
