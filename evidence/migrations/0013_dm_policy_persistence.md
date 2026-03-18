# Migration Validation Evidence - 0013_dm_policy_persistence

## Document Metadata

- Doc ID: migration-validation-0013-dm-policy-persistence
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-18
- Source of truth: `evidence/migrations/0013_dm_policy_persistence.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0013_dm_policy_persistence.sql`.
- Primary edit location: update when DM policy persistence evidence changes.
- Latest meaningful change: 2026-03-18 added delivery-pass evidence for restart-stable persisted DM privacy policy authority.

## Migration Metadata

- Migration ID: `0013_dm_policy_persistence`
- Owner: Maintainers
- Date (UTC): 2026-03-18
- Environment tested: local dev (Windows) + CI workflow (`migration-evidence-check`, `rust-check`, `rust-coverage-gate`)
- Commit SHA: `1e16e6d`
- PR number (or CI run ID): `PR #42` / `23242910720`
- Generated at (UTC): `2026-03-18T12:05:00Z`

## Forward Validation

- Command(s) executed: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`
- Expected outcome: DM privacy policy writes persist into `dm_policies` and subsequent reads use DB authority instead of process-memory state.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation for `services/api-rs` in the PR #42 delivery session.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test --all-features`
- Expected outcome: repeated startup and policy updates leave one authoritative row per identity and restart coverage remains green.
- Actual outcome: pass.
- Evidence path: local CLI output from repeated API test execution in the delivery session.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive schema in application-level validation pass.
- Expected outcome: recovery uses restore/runbook procedures if rollback is ever needed.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md`

## Data Integrity Verification

- Constraints/indexes verified: `dm_policies` primary key, `dm_policies_identity_fk`, `dm_policies_inbound_policy_check`, `dm_policies_offline_delivery_mode_check`.
- Row-count or key invariants checked: persisted DM policy survives restart and remains keyed by authenticated identity only.
- Evidence path: `services/api-rs/src/tests/integration/dm_policy_tests.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration moves DM privacy policy authority out of volatile memory without widening server-side DM payload retention.
