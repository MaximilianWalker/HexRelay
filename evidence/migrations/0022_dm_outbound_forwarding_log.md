# Migration Validation Evidence - 0022_dm_outbound_forwarding_log

## Document Metadata

- Doc ID: migration-validation-0022-dm-outbound-forwarding-log
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `evidence/migrations/0022_dm_outbound_forwarding_log.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0022_dm_outbound_forwarding_log.sql`.
- Primary edit location: update when outbound server DM forwarding persistence changes.
- Latest meaningful change: 2026-05-11 added the persisted outbound forwarding log for server DM envelope forwarding.

## Migration Metadata

- Migration ID: `0022_dm_outbound_forwarding_log`
- Owner: Maintainers
- Date (UTC): 2026-05-11
- Environment tested: local dev (Windows) with targeted API checks and PR CI `migration-evidence-check`
- Commit SHA: pending current PR
- PR number (or CI run ID): `PR #108`
- Generated at (UTC): `2026-05-11T08:15:00Z`

## Forward Validation

- Command(s) executed: `cargo test -p api-rs dm_fanout`, `cargo test -p api-rs outbound_forward`, `cargo test -p api-rs`, `cargo clippy -p api-rs --all-targets -- -D warnings`, `bash scripts/validators/dm-transport-policy.sh`.
- Expected outcome: outbound server DM forwarding attempts persist encrypted envelope metadata in `dm_outbound_forwarding_log` without introducing direct user-to-user transport state.
- Actual outcome: pass locally; PR CI identified this evidence artifact was required and is satisfied by this file.
- Evidence path (logs/artifacts): local CLI validation and `evidence/migrations/0022_dm_outbound_forwarding_log.md`.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test -p api-rs outbound_forward`, `cargo test -p api-rs dm_fanout`.
- Expected outcome: migration creates the forwarding log and indexes with `IF NOT EXISTS`; repeated application is guarded by `schema_migrations`.
- Actual outcome: pass.
- Evidence path: `services/api-rs/migrations/0022_dm_outbound_forwarding_log.sql`, `services/api-rs/src/infra/db/repos/dm_outbound_forward_repo.rs`, `services/api-rs/src/tests/integration/dm_fanout_tests.rs`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore a database backup taken before the migration, or drain outbound forwarding records before dropping the additive table during a controlled rollback.
- Expected outcome: current runtime should not run server DM forwarding retry logic against a schema without `dm_outbound_forwarding_log`.
- Actual outcome: acknowledged; additive forwarding persistence table.
- Evidence path: `docs/architecture/04-runtime-contract.md`, `docs/architecture/07-data-lifecycle.md`.

## Data Integrity Verification

- Constraints/indexes verified: primary key `(sender_identity_id, destination_server_id, message_id)`, sender identity foreign key, forwarding state check, sender-created index, and destination-state index.
- Row-count or key invariants checked: each sender, destination server, and encrypted message has one durable outbound forwarding record; server forwarding stores ciphertext and delivery metadata only.
- Evidence path: `services/api-rs/migrations/0022_dm_outbound_forwarding_log.sql`.

## Sign-off

- Reviewer: Codex agent (PR #108 CI correction pass)
- Decision: pass
- Notes: This migration supports server relay persistence only; it does not add user-to-user LAN/WAN DM transport or server-readable DM content.
