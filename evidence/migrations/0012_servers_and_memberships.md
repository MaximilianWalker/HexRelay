# Migration Validation Evidence - 0012_servers_and_memberships

## Document Metadata

- Doc ID: migration-validation-0012-servers-and-memberships
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-18
- Source of truth: `evidence/migrations/0012_servers_and_memberships.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0012_servers_and_memberships.sql`.
- Primary edit location: update when validation evidence for persisted server memberships changes.
- Latest meaningful change: 2026-03-18 added delivery-pass evidence for persisted `/v1/servers` membership authority and trusted `same_server` DM checks.

## Migration Metadata

- Migration ID: `0012_servers_and_memberships`
- Owner: Maintainers
- Date (UTC): 2026-03-18
- Environment tested: local dev (Windows) + CI workflow (`migration-evidence-check`, `rust-check`, `rust-coverage-gate`)
- Commit SHA: `1e16e6d`
- PR number (or CI run ID): `PR #42` / `23242910720`
- Generated at (UTC): `2026-03-18T12:05:00Z`

## Forward Validation

- Command(s) executed: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`
- Expected outcome: startup migration creates persisted `servers` and `server_memberships`, `/v1/servers` reads authenticated memberships from DB, and `same_server` DM authorization uses trusted shared membership state.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation for `services/api-rs` in the PR #42 delivery session.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test --all-features`
- Expected outcome: repeated `connect_and_prepare` runs leave `servers` and `server_memberships` intact without duplicate-key failures, and DB-backed directory/DM integration tests remain green.
- Actual outcome: pass.
- Evidence path: local CLI output from repeated API test execution in the delivery session.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive schema in application-level validation pass.
- Expected outcome: recovery follows documented database restore procedures rather than destructive in-place rollback.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md`

## Data Integrity Verification

- Constraints/indexes verified: `servers` primary key, `server_memberships` composite primary key, `server_memberships_server_fk`, `server_memberships_identity_fk`, `server_memberships_identity_joined_idx`.
- Row-count or key invariants checked: authenticated server listings remain identity-scoped and trusted `same_server` policy checks allow only when both identities share persisted membership rows.
- Evidence path: `services/api-rs/src/tests/integration/directory_tests.rs`, `services/api-rs/src/tests/integration/dm_connectivity_tests.rs`, `services/api-rs/src/tests/integration/dm_fanout_tests.rs`, `services/api-rs/src/tests/integration/dm_lan_discovery_tests.rs`, `services/api-rs/src/tests/integration/dm_parallel_dial_tests.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration closes the trusted shared-membership authority gap for `/v1/servers` and `same_server` DM policy evaluation.
