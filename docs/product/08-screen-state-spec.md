# HexRelay MVP Screen and State Spec

## Document Metadata

- Doc ID: screen-state-spec
- Owner: Product and design maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/product/08-screen-state-spec.md`

## Quick Context

- Purpose: define screen-level states so Web/API/Core teams implement the same behavior.
- Primary edit location: update when a screen flow or failure mode changes.
- Latest meaningful change: 2026-03-04 execution-hardening pass added MVP-wide state coverage.

## Core Screens

- Identity Create/Import
- Server Join (invite)
- Contacts Hub and friend request flows
- DM Workspace
- Servers Hub and Server Workspace
- Voice/Call overlay and screen share controls
- Attachment transfer flow
- Migration Export/Import flow
- Observability and SLO review surface

## State Matrix

| Screen | Required states |
|---|---|
| Identity Create/Import | loading, success, invalid_key, storage_failed |
| Server Join | loading, fingerprint_mismatch, invite_invalid, invite_expired, invite_exhausted, success |
| Contacts Hub | loading, empty, search_no_results, friend_request_pending, friend_request_inbound, error |
| DM Workspace | loading, empty, blocked, policy_denied, send_failed_retryable, reconnecting |
| Servers Hub | loading, empty, search_no_results, permission_denied, error |
| Server Workspace | loading, channel_empty, permission_denied, reconnecting, error |
| Voice/Screen Share | connecting, connected, reconnecting, quality_degraded, ended, error |
| Attachment Transfer | loading, upload_progress, success, retryable_failure, policy_denied |
| Migration | export_running, import_running, conflict_review, reconcile_running, completed, failed |
| Observability/SLO Review | loading, degraded, breached, recovered |

## Policy-Driven States

- DM inbound policy denied (`friends_only`, `same_server`, `anyone`) must surface deterministic reason and settings shortcut.
- Friend request mediation must not reveal raw identity bootstrap data before acceptance.
- DM offline outbox retries must surface deterministic states (`queued`, `retrying`, `delivered`, `failed`).

## Related Documents

- `docs/product/07-ui-navigation-spec.md`
- `docs/planning/iterations/01-sprint-board.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/planning/iterations/03-sprint-board.md`
- `docs/planning/iterations/04-sprint-board.md`
