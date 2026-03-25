# Migration Validation Evidence - 0017_server_channels_and_messages

## Document Metadata

- Doc ID: migration-validation-0017-server-channels-and-messages
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-25
- Source of truth: `evidence/migrations/0017_server_channels_and_messages.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0017_server_channels_and_messages.sql`.
- Primary edit location: update when validation evidence for persisted server-channel storage changes.
- Latest meaningful change: 2026-03-25 added delivery-pass evidence for persisted server-channel message listing/creation, same-channel reply validation, and same-server mention validation.

## Migration Metadata

- Migration ID: `0017_server_channels_and_messages`
- Owner: Maintainers
- Date (UTC): 2026-03-25
- Environment tested: local dev (Windows) + GitHub Actions (`migration-evidence-check`, `rust-check`, `rust-coverage-gate`)
- Commit SHA: `64f9456`
- PR number (or CI run ID): `PR #66` / `23564014637`
- Generated at (UTC): `2026-03-25T21:15:00Z`

## Forward Validation

- Command(s) executed: `cargo fmt --all`, `cargo clippy -p api-rs --all-targets -- -D warnings`, `cargo test -p api-rs server_channel_messages -- --nocapture`, `cargo test -p api-rs -- --nocapture`, `bash scripts/validate-contract-parity.sh`, `bash scripts/validate-docs-index-freshness.sh`
- Expected outcome: startup migration creates persisted server-channel tables, runtime `GET/POST /v1/servers/{server_id}/channels/{channel_id}/messages` uses DB-backed channel/message storage, and reply/mention validation is enforced against persisted channel/server membership state.
- Actual outcome: pass.
- Evidence path (logs/artifacts): local CLI validation in the PR #66 delivery session; CI run `23564014637` for contract/docs and Rust validation jobs.

## Idempotency and Re-run Check

- Re-run command(s): `cargo test -p api-rs server_channel_messages -- --nocapture`, `cargo test -p api-rs -- --nocapture`
- Expected outcome: repeated startup and reseed flows leave additive channel/message schema intact without duplicate-key failures, and channel message integration tests remain green under rerun.
- Actual outcome: pass.
- Evidence path: local CLI rerun output captured during PR #66 delivery.

## Rollback/Recovery Simulation

- Rollback or restore command(s): N/A for additive schema in application-level validation pass.
- Expected outcome: recovery uses documented database restore procedures instead of destructive in-place rollback.
- Actual outcome: acknowledged; additive schema-only change.
- Evidence path: `docs/operations/01-mvp-runbook.md`

## Data Integrity Verification

- Constraints/indexes verified: `server_channels` primary key, `server_channels_server_fk`, `server_channels_kind_check`, `server_channels_server_created_idx`, `server_channel_messages` primary key, `server_channel_messages_channel_fk`, `server_channel_messages_author_fk`, `server_channel_messages_reply_fk`, `server_channel_messages_channel_seq_unique`, `server_channel_messages_channel_seq_idx`, `server_channel_messages_reply_idx`, `server_channel_message_mentions` composite primary key, `server_channel_message_mentions_message_fk`, `server_channel_message_mentions_identity_fk`, `server_channel_message_mentions_identity_idx`.
- Row-count or key invariants checked: authenticated server members can list/create only within channels scoped to their persisted server membership; non-members receive `server_access_denied`; unknown channels return `channel_not_found`; reply targets must exist in the same channel; mention targets must exist in persisted memberships for the same server.
- Evidence path: `services/api-rs/src/tests/integration/server_channel_messages_tests.rs`, `services/api-rs/src/infra/db/repos/server_channels_repo.rs`, `services/api-rs/src/transport/http/handlers/server_channels.rs`.

## Sign-off

- Reviewer: OpenCode agent (delivery validation pass)
- Decision: pass
- Notes: This migration intentionally scopes the first `T4.3.1` runtime slice to persisted server-channel message read/create semantics only; edit/delete, realtime fanout, and richer permission layers remain deferred.
