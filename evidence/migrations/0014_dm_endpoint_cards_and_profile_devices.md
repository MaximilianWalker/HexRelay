# Migration Validation Evidence - 0014_dm_endpoint_cards_and_profile_devices

## Document Metadata

- Doc ID: migration-validation-0014-dm-endpoint-cards-and-profile-devices
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-18
- Source of truth: `evidence/migrations/0014_dm_endpoint_cards_and_profile_devices.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0014_dm_endpoint_cards_and_profile_devices.sql`.
- Primary edit location: update when persisted endpoint-card or profile-device evidence changes.
- Latest meaningful change: 2026-03-18 added delivery-pass evidence for restart-stable direct endpoint cards and active-device records.

## Migration Metadata

- Migration ID: `0014_dm_endpoint_cards_and_profile_devices`
- Owner: Maintainers
- Date (UTC): 2026-03-18
- Environment tested: local dev (Windows) + CI workflow (`migration-evidence-check`, `rust-check`, `rust-coverage-gate`)
- Commit SHA: `1e16e6d`
- PR number (or CI run ID): `PR #42` / `23242910720`
- Generated at (UTC): `2026-03-18T12:05:00Z`

## Forward Validation

- Command(s) executed: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`
- Expected outcome: endpoint-card registration/revocation and profile-device heartbeat state persist across app restart when DB storage is configured.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation for `services/api-rs` in the PR #42 delivery session.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test --all-features`
- Expected outcome: repeated startup and repeated endpoint/device writes update existing rows deterministically without duplicate-key drift.
- Actual outcome: pass.
- Evidence path: local CLI output from repeated API test execution in the delivery session.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive schema in application-level validation pass.
- Expected outcome: recovery uses documented restore procedures if rollback is required.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md`

## Data Integrity Verification

- Constraints/indexes verified: `dm_endpoint_cards` composite primary key and identity FK, `dm_endpoint_cards_identity_expiry_idx`, `dm_profile_devices` composite primary key and identity FK, `dm_profile_devices_identity_seen_idx`.
- Row-count or key invariants checked: endpoint-card winner selection and active-device availability remain stable after restart for the same authenticated identity.
- Evidence path: `services/api-rs/src/tests/integration/dm_policy_tests.rs`, `services/api-rs/src/tests/integration/dm_parallel_dial_tests.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration preserves direct-only connectivity metadata across restart while keeping ownership on user/device-controlled state.
