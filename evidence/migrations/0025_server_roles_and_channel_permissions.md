# Migration Validation Evidence - 0025_server_roles_and_channel_permissions

## Document Metadata

- Doc ID: migration-validation-0025-server-roles-and-channel-permissions
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-13
- Source of truth: `evidence/migrations/0025_server_roles_and_channel_permissions.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0025_server_roles_and_channel_permissions.sql`.
- Primary edit location: update when server role, membership-role, or channel permission constraints change.
- Latest meaningful change: 2026-05-13 added server role and per-channel read/send permission schema evidence.

## Migration Metadata

- Migration ID: `0025_server_roles_and_channel_permissions`
- Owner: Maintainers
- Date (UTC): 2026-05-13
- Environment tested: local dev with Postgres-backed API integration tests
- Commit SHA: current branch commit containing this evidence
- PR number (or CI run ID): local validation before PR
- Generated at (UTC): `2026-05-13T05:07:10Z`

## Forward Validation

- Command(s) executed: `cargo test -p api-rs channel_listing_honors_configured_role_read_permissions --all-features`; `cargo test -p api-rs channel_role_send_permission_gates_server_channel_message_creation --all-features`; `cargo test -p api-rs server_role_assignment_is_scoped_to_matching_server_membership --all-features`; `RUST_TEST_THREADS=1 cargo test -p api-rs --all-features`.
- Expected outcome: migration applies through `connect_and_prepare`, server role and channel permission constraints exist, configured channel read permissions filter channel listings, and configured send permissions gate message creation.
- Actual outcome: pass. Focused API integration tests validated role-scoped channel listing, send-permission denial/allow behavior, and cross-server role-assignment rejection; the full API suite passed with the same serialized Postgres test setting used in CI.
- Evidence path (logs/artifacts): local CLI validation and this file.

## Idempotency and Re-run Check

- Re-run command(s): migration application through `connect_and_prepare` in each focused API integration test and the full serialized API test suite.
- Expected outcome: migration is recorded in `schema_migrations`; additive tables and indexes use idempotent creation where possible; the existing `server_channels(server_id, channel_id)` unique constraint is guarded by a catalog check.
- Actual outcome: pass through focused API integration tests and `RUST_TEST_THREADS=1 cargo test -p api-rs --all-features`.
- Evidence path: `services/api-rs/migrations/0025_server_roles_and_channel_permissions.sql`.

## Rollback/Recovery Simulation

- Rollback or restore command(s): restore a database backup taken before migration, or drop `server_channel_role_permissions`, `server_membership_roles`, `server_roles`, and `server_channels_server_channel_unique` during controlled rollback before reapplying from backup.
- Expected outcome: existing server/channel/message rows are not rewritten by the forward migration; removing the additive role tables reverts to pre-role member-only authorization behavior.
- Actual outcome: additive schema only; no stored server-channel message or membership data is rewritten.
- Evidence path: repository migration and focused authorization tests.

## Data Integrity Verification

- Constraints/indexes verified: `server_roles_server_fk`, `server_roles_server_name_unique`, `server_roles_server_role_unique`, `server_membership_roles_membership_fk`, `server_membership_roles_role_fk`, `server_channel_role_permissions_channel_fk`, `server_channel_role_permissions_role_fk`, read-required send/manage checks, and role/channel lookup indexes.
- Row-count or key invariants checked: cross-server role assignment is rejected by `server_membership_roles_role_fk`; configured channel permissions deny read/send without altering existing server membership rows.
- Evidence path: `services/api-rs/src/tests/integration/authorization_tests.rs` and `services/api-rs/src/tests/integration/server_channel_messages_tests.rs`.

## Sign-off

- Reviewer: Codex agent
- Decision: pass
- Notes: This migration is a backend/API schema prerequisite for server-channel role enforcement and does not add product UI or UX behavior.
