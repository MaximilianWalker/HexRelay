# Migration Validation Evidence - 0023_dm_outbound_forwarding_retry_schedule

## Document Metadata

- Doc ID: migration-validation-0023-dm-outbound-forwarding-retry-schedule
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `evidence/migrations/0023_dm_outbound_forwarding_retry_schedule.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0023_dm_outbound_forwarding_retry_schedule.sql`.
- Primary edit location: update when outbound server-node DM retry scheduling changes.
- Latest meaningful change: 2026-05-11 added `next_attempt_at` scheduling for bounded outbound server-node DM forwarding retries.

## Migration Metadata

- Migration ID: `0023_dm_outbound_forwarding_retry_schedule`
- Owner: Maintainers
- Date (UTC): 2026-05-11
- Environment tested: local dev (Windows) with targeted API checks and PR CI `migration-evidence-check`
- Commit SHA: pending current PR
- PR number (or CI run ID): `PR #108`
- Generated at (UTC): `2026-05-11T08:15:00Z`

## Forward Validation

- Command(s) executed: `cargo test -p api-rs outbound_forward`, `cargo test -p api-rs dm_fanout`, `cargo test -p api-rs`, `cargo clippy -p api-rs --all-targets -- -D warnings`, `bash scripts/validate-dm-transport-policy.sh`.
- Expected outcome: queued and failed outbound forwarding records get a due retry timestamp, and retry selection can find eligible server-node forwarding attempts deterministically.
- Actual outcome: pass locally; PR CI identified this evidence artifact was required and is satisfied by this file.
- Evidence path (logs/artifacts): local CLI validation and `evidence/migrations/0023_dm_outbound_forwarding_retry_schedule.md`.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test -p api-rs outbound_forward`.
- Expected outcome: `next_attempt_at` can be added idempotently, existing queued or failed records without a schedule are made due once, and repeated migration application is guarded by `schema_migrations`.
- Actual outcome: pass.
- Evidence path: `services/api-rs/migrations/0023_dm_outbound_forwarding_retry_schedule.sql`, `services/api-rs/src/infra/db/repos/dm_outbound_forward_repo.rs`, `services/api-rs/src/tests/integration/dm_fanout_tests.rs`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore a database backup taken before the migration, or pause retry workers before removing retry scheduling data during a controlled rollback.
- Expected outcome: current runtime retry logic should not run against a schema without `next_attempt_at`.
- Actual outcome: acknowledged; additive nullable column and retry index.
- Evidence path: `docs/architecture/04-runtime-contract.md`, `docs/architecture/07-data-lifecycle.md`.

## Data Integrity Verification

- Constraints/indexes verified: nullable `next_attempt_at` column and retry index over `(forwarding_state, next_attempt_at, attempt_count)`.
- Row-count or key invariants checked: queued and failed records without a retry schedule become due without changing sender, destination node, message, recipient, ciphertext, or delivery cursor values.
- Evidence path: `services/api-rs/migrations/0023_dm_outbound_forwarding_retry_schedule.sql`.

## Sign-off

- Reviewer: Codex agent (PR #108 CI correction pass)
- Decision: pass
- Notes: Retry scheduling remains server-node forwarding infrastructure and does not expose new UX behavior or direct user-to-user transport.
