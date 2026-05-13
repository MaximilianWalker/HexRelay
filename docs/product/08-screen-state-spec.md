# HexRelay MVP Screen and State Spec

## Document Metadata

- Doc ID: screen-state-spec
- Owner: Product and design maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-13
- Source of truth: `docs/product/08-screen-state-spec.md`

## Quick Context

- Purpose: define screen-level states so Web/API/Core teams implement the same behavior.
- Primary edit location: update when a screen flow or failure mode changes.
- Latest meaningful change: 2026-05-13 added an approval-pending server-channel optimistic send proposal without authorizing UI implementation.

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

## UX Approval Gate

- No UX flow, copy, control, or behavior change may be implemented until the user explicitly consents to it.

## State Matrix

| Screen | Required states |
|---|---|
| Identity Create/Import | loading, success, invalid_key, storage_failed |
| Server Join | loading, fingerprint_mismatch, invite_invalid, invite_expired, invite_exhausted, success |
| Contact Invite Share | idle, creating, created (link visible), error |
| Contact Invite Redeem | idle, redeeming, success (friend request created), invite_invalid, invite_expired, invite_exhausted, self_invite |
| Contacts Hub | loading, empty, search_no_results, friend_request_pending, friend_request_inbound, error |
| DM Workspace | loading, empty, blocked, policy_denied, send_failed_retryable, reconnecting |
| Servers Hub | loading, empty, search_no_results, permission_denied, error |
| Server Workspace | loading, channel_empty, permission_denied, reconnecting, error |
| Voice/Screen Share | connecting, connected, reconnecting, quality_degraded, ended, error |
| Attachment Transfer | loading, upload_progress, success, retryable_failure, policy_denied |
| Migration | export_running, import_running, conflict_review, reconcile_running, completed, failed |
| Observability/SLO Review | loading, degraded, breached, recovered |

## Server Channel Optimistic Send Proposal

Status: approval_pending. This section is a plan-only proposal for `T4.3.2`; no product UI behavior is approved for implementation until the user explicitly accepts the flow, copy, controls, and behavior below.

Scope: server-channel message create only. Edit and delete actions should stay confirmation-driven until separately approved.

Flow:

1. Pressing send with non-empty content adds a local pending row at the bottom of the active channel and clears the composer.
2. The pending row shows the exact submitted content, author identity, local timestamp, and `Sending` state.
3. API success replaces the pending row with the persisted server-channel message returned by the create route.
4. The websocket `channel.message.created` event is idempotent against the persisted message id; it must not create a duplicate row after API success.
5. API failure converts the pending row to `Could not send` and keeps the original text available for retry or edit.
6. Reconnect or refresh reconciles from persisted channel history. Locally pending rows without durable success must not be auto-sent again without an explicit user retry.

Controls:

- Primary send control: existing composer send action.
- Failed-row controls: `Retry`, `Edit`, `Discard`.
- Retry resubmits the preserved text once; Edit restores it to the composer; Discard removes only the local failed row.

Copy baseline:

- Pending label: `Sending`
- Durable success label for assistive text: `Sent`
- Failure label: `Could not send`
- Control labels: `Retry`, `Edit`, `Discard`

Behavior constraints:

- Do not expose backend dispatch summaries as user-visible delivery or read state.
- Do not imply recipient delivery from websocket fanout; server-channel history persistence is the durable truth.
- Do not add unread/read receipt behavior in this scope.
- Do not introduce DM transport concepts, endpoint cards, preflight checks, or node-bypassing language.

## Policy-Driven States

- DM inbound policy denied (`friends_only`, `same_server`, `anyone`) must surface deterministic reason and settings shortcut.
- Friend request mediation must not reveal raw identity bootstrap data before acceptance.
- DM offline outbox retries must surface deterministic states (`queued`, `retrying`, `delivered`, `failed`).

## DM Message Delivery Indicators

- Approved direction: use compact gaming-style HUD pips rather than WhatsApp-style checkmarks.
- Default chat rows stay visually quiet like Discord; delivery detail appears beside the timestamp as icon/pip states, with text available on hover, focus, or long-press.
- Indicators must be accessible: do not rely on color alone, and expose clear labels such as `Sending`, `Sent`, `Delivered`, `Read`, and `Failed`.
- `Delivered` must never imply `Read`. Delivery is device receipt of the encrypted envelope; read state requires a separate explicit `dm.message.read` receipt.
- Participant-visible read receipts must respect the reader's privacy setting; when receipts are disabled, read-state sync may remain limited to the reader's own profile devices.
- Delivery indicators must not introduce DM preflight, node-bypassing connection controls, troubleshooting wizard behavior, endpoint cards, or node-bypassing DM transport concepts.

| UI state | Visual direction | Backend truth |
|---|---|---|
| Sending | Dim animated pip or pulse | Local client send/envelope preparation is in progress |
| Sent | One muted steel/grey HUD pip | API durably accepted the encrypted envelope into server-node DM history |
| Delivered | Two muted steel/grey linked HUD pips | At least one recipient profile device acked `dm.envelope.dispatched` |
| Read | Two linked HUD pips with cyan/blue active treatment plus non-color affordance | Target-state `dm.message.read` receipt with participant-visible scope, not the delivery ack |
| Queued | Amber subdued pip or clock-like HUD accent | Durable acceptance exists, but no recipient-device ack is known yet |
| Failed | Red broken pip plus retry affordance | Send failed or became unrecoverable |

### Group DM Aggregation

- Group DMs show aggregate counts by default: `3/5 delivered`, `2/5 read`.
- Read counts include only participant-visible read receipts; self-device-only read-state sync must not appear as another participant's read receipt.
- Hover/focus/long-press detail may show recipient breakdown by display name and state.
- The message row should avoid noisy per-recipient avatars by default; breakdown belongs in a compact popover or message details panel.

### Copy Baseline

- Use plain labels for accessibility and tooltips: `Sending`, `Sent`, `Delivered`, `Read`, `Queued`, `Failed`.
- Optional themed microcopy may be used in expanded details only if it remains understandable, for example `Node accepted`, `Device received`, or `Read by Alex`.
- Avoid overloaded gamer slang for core labels; the gaming feel should come from motion, color, spacing, sound, and icon treatment rather than unclear terminology.

## Related Documents

- `docs/product/07-ui-navigation-spec.md`
- `docs/planning/iterations/01-sprint-board.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/planning/iterations/03-sprint-board.md`
- `docs/planning/iterations/04-sprint-board.md`
