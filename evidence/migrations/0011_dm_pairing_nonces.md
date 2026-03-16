# Migration Validation Evidence - 0011_dm_pairing_nonces

## Document Metadata

- Doc ID: migration-validation-0011-dm-pairing-nonces
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-16
- Source of truth: `evidence/migrations/0011_dm_pairing_nonces.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0011_dm_pairing_nonces.sql`.
- Primary edit location: update when migration validation evidence for this migration changes.
- Latest meaningful change: 2026-03-16 added replay-protection persistence evidence for DM pairing nonces.

## Migration Metadata

- Migration ID: `0011_dm_pairing_nonces`
- Owner: Maintainers
- Date (UTC): 2026-03-16
- Environment tested: local dev (Windows) + CI workflow (`rust-check`, `migration-evidence-check`)
- Commit SHA: `4176e30`
- PR number (or CI run ID): `PR #37`
- Generated at (UTC): `2026-03-16T16:00:00Z`

## Forward Validation

- Command(s) executed: `cargo fmt --all && cargo test -p api-rs dm_ && cargo clippy -p api-rs --all-targets -- -D warnings`
- Expected outcome: migration is registered by `connect_and_prepare`; DM pairing import consumes nonces through DB-backed replay store when DB is configured, with no API regression.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI output from the validation run in this implementation session.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test -p api-rs dm_`
- Expected outcome: migration remains idempotent (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`) and DM integration tests remain green.
- Actual outcome: pass.
- Evidence path: local CLI output from repeated DM integration test run in this session.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive migration in application-level validation pass.
- Expected outcome: no destructive rollback required; recovery follows restore procedures in runbook.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md` backup/restore procedures.

## Data Integrity Verification

- Constraints/indexes verified: `dm_pairing_nonces` primary key on `nonce` and expiry index `idx_dm_pairing_nonces_expires_at`.
- Row-count or key invariants checked: nonce replay semantics are single-consume; repeated import of same envelope returns `pairing_replayed` while valid first import succeeds.
- Evidence path: `services/api-rs/src/transport/http/handlers/dm.rs` and `services/api-rs/src/tests/integration/dm_pairing_tests.rs`.

## Sign-off

- Reviewer: OpenCode agent (implementation pass)
- Decision: pass
- Notes: This migration closes readiness finding for process-restart replay-window regression by moving pairing nonce replay state to durable DB storage when available.
