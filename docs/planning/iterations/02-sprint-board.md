# HexRelay Iteration 2 Sprint Board

## Document Metadata

- Doc ID: iteration-02-sprint-board
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-16
- Source of truth: `docs/planning/iterations/02-sprint-board.md`
- Board status: in_progress

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-16 extended networking backlog with profile-device eventual-sync tasks for DM and server communication paths.

## Iteration Scope

Scope: Iteration 2 (Weeks 4-6) from `docs/product/01-mvp-plan.md`.

## Goals

- Ship social graph and presence as stable primitives.
- Deliver DM/group DM and guild/channel messaging with permission enforcement.
- Ship global/shared-server user discovery and E2EE baseline for 1:1 and group DMs.
- Establish reliable realtime fanout patterns used by later voice and federation work.

## Status Legend

- `planned`: not started
- `in_progress`: active implementation
- `done`: acceptance criteria met
- `blocked`: cannot proceed due to unresolved dependency

## Effort Scale

- S: 0.5-1 day
- M: 1-2 days
- L: 2-4 days
- XL: 4-6 days

## Backlog

| ID | Task | Epic/Story | Effort | Owner | Dependencies | Acceptance Criteria |
|---|---|---|---|---|---|---|
| T3.1.1 | Implement friend request state machine and DB constraints | E3 / S3.1 | L | API | T2.3.1, T2.4.1 | Requests support pending/accepted/declined/cancelled with invariant tests and mediated request routing |
| T3.1.2 | Build friends list UI and request actions | E3 / S3.1 | M | Web | T3.1.1 | Users can send/respond to requests from UI with optimistic updates; no raw key/profile-identifying data is shown pre-acceptance |
| T3.1.3 | Implement user contact invite token create/redeem APIs | E3 / S3.1 | M | API | T3.1.1 | Users can create expiring contact invites and redeem valid tokens; invalid/expired/exhausted cases return explicit errors |
| T3.1.4 | Implement contact invite share/scan UX (link + QR) | E3 / S3.1 | M | Web | T3.1.3 | User can share invite link/QR and recipient can redeem from UI with deterministic success/error states |
| T3.1.5 | Enforce mediated identity bootstrap on friend acceptance | E3 / S3.1 | M | API | T3.1.1 | Bootstrap identity material is shared only after acceptance and is blocked on pending/declined states |
| T3.2.1 | Implement block/mute logic and fanout filters | E3 / S3.2 | M | API | T3.1.1 | Blocked/muted users are excluded from delivery paths as defined |
| T3.3.1 | Implement presence service with Redis ephemeral state | E3 / S3.3 | L | Realtime | T1.1.2 | Presence transitions propagate within p95 <= 1s and recover correctly after reconnect in integration tests |
| T3.3.2 | Add profile-device presence convergence and late-device hydration | E3 / S3.3 | M | Realtime/Core | T3.3.1, T4.0.3 | Presence state converges across all active profile devices and hydrates missed transitions on later-active devices |
| T3.4.1 | Implement global user discovery index and shared-server query | E3 / S3.4 | L | API | T3.1.1 | Discovery returns only permitted profiles, excludes blocked users, and enforces rate-limit/denylist controls in policy tests |
| T4.0.1 | Define shared communication layer interfaces and policy engine boundary | E4 / S4.1 | M | Core | T3.3.1 | Common communication interface supports `dm_direct` and `server_channel` modes with deterministic policy routing |
| T4.0.2 | Implement transport adapter boundaries (`DirectPeerTransport`, `NodeClientTransport`) | E4 / S4.1 | L | Core | T4.0.1 | Existing call paths route through adapter interfaces without behavior regression |
| T4.0.3 | Implement shared session provenance and reason-code taxonomy | E4 / S4.1 | M | Core | T4.0.1 | Both DM and server paths emit stable provenance + reason-code outputs |
| T4.1.1 | Implement client-side DM/group DM thread model and history pagination | E4 / S4.1 | L | Core | T3.1.1, T4.0.2 | DM threads support cursor pagination and unread markers without guild server persistence |
| T4.1.2 | Implement DM privacy policy defaults and user override settings | E4 / S4.1 | M | Core | T4.1.1 | Incoming DM policy defaults to friends-only; user can opt into same-server or anyone modes |
| T4.1.3 | Enforce direct-only DM transport and infra-policy CI guardrails | E4 / S4.1 | M | Core | T4.1.1 | DM transport rejects STUN/TURN/relay fallback paths and CI policy checks fail on forbidden config/callsites |
| T4.1.4 | Implement signed out-of-band DM pairing envelope + QR/short-code bootstrap | E4 / S4.1 | L | Core/Web | T3.1.4, T4.1.3 | Contact pairing works via signed envelope with replay/expiry checks and without backend rendezvous dependency |
| T4.1.5 | Implement DM connectivity preflight and deterministic troubleshooter | E4 / S4.1 | M | Core/Web | T4.1.4 | Failed direct connections map to deterministic reason codes with actionable remediation guidance |
| T4.1.6 | Implement LAN discovery fast path for DM direct connect (mDNS/multicast) | E4 / S4.1 | L | Realtime/Core | T4.1.5 | Same-LAN peers discover/connect through local-only traffic and improved connect latency |
| T4.1.7 | Implement WAN direct-connect wizard (UPnP/NAT-PMP + manual mapping) | E4 / S4.1 | L | Core/Web | T4.1.5 | Wizard emits deterministic outcomes (`success`, `manual_required`, `network_incompatible`) with verification steps |
| T4.1.8 | Implement multi-endpoint DM endpoint cards and parallel dial orchestration | E4 / S4.1 | M | Core | T4.1.4, T4.1.6 | Parallel dial improves connect success and deterministically cancels non-winning dials |
| T4.1.9 | Implement DM active-device profile fanout semantics | E4 / S4.1 | M | Core/Realtime | T4.1.8 | One incoming DM payload fanouts to all currently active devices linked to recipient profile |
| T4.1.10 | Implement DM late-device catch-up and per-device cursor dedupe | E4 / S4.1 | L | Core | T4.1.9 | Devices activated after first receive replay missed payloads and converge deterministically |
| T4.2.1 | Implement guild/channel/role schema | E4 / S4.2 | L | API | T4.1.1 | Roles and channel membership constraints are enforced in DB/API |
| T4.2.2 | Build server/channel management UI | E4 / S4.2 | M | Web | T4.2.1 | Owners/admins can create channels and assign base roles |
| T4.3.1 | Implement server-channel message CRUD/reply/mention endpoints | E4 / S4.3 | XL | API | T4.2.1 | Server channels support create/edit/delete/reply/mention with audit-safe events |
| T4.3.2 | Add websocket event fanout and optimistic UI for server channels | E4 / S4.3 | L | Realtime | T4.3.1, T3.3.1, T4.0.2 | Clients receive strictly ordered server-channel events; reconnect tests show no lost/duplicated events |
| T4.3.3 | Route server-channel and presence communication through `NodeClientTransport` adapter | E4 / S4.3 | M | Realtime/Core | T4.0.2, T4.3.2 | Server communication is adapterized with no DM policy leakage or regression |
| T4.3.4 | Implement server-channel profile-device fanout and late-device hydration | E4 / S4.3 | L | Realtime/Core | T4.3.3, T3.3.2 | Server-channel and presence events converge across all profile devices, including devices that reconnect later |
| T4.4.1 | Add permission middleware and authorization tests | E4 / S4.4 | L | API | T4.2.1, T4.3.1 | Permission bypass attempts fail and are covered in tests |
| T4.5.1 | Implement E2EE DM key exchange/session bootstrap for 1:1 DMs | E4 / S4.5 | L | Core | T4.1.1 | Peers establish encrypted sessions with verifiable identity keys |
| T4.5.2 | Implement E2EE DM encrypt/decrypt flow with key rotation (1:1) | E4 / S4.5 | XL | Core | T4.5.1 | 1:1 DM messages travel over direct user-to-user channels, decrypt correctly on clients, and enqueue encrypted local outbox retries when recipient is offline |
| T4.5.3 | Implement group DM E2EE session bootstrap and membership key updates | E4 / S4.6 | XL | Core | T4.5.2 | Group session keys update on member add/remove and old members cannot decrypt new traffic |
| T4.5.4 | Implement group DM E2EE encrypt/decrypt and failure recovery paths | E4 / S4.6 | XL | Core | T4.5.3 | Group DM payloads use direct user-to-user transport; rekey and missing-key failures are test-covered |
| T4.6.1 | Implement `Servers Hub` UI surface from navigation spec | E4 / S4.2 | L | Web | T4.2.2 | Search/filter/pin actions work and deep-link into server workspace |
| T4.6.2 | Implement `Contacts Hub` UI surface from navigation spec | E4 / S4.1 | L | Web | T3.1.2 | Search/filter/open-DM actions work and state persists per user |
| T4.6.3 | Implement dual server navigation modes and burger persistence | E4 / S4.2 | L | Web | T4.6.1 | Topbar supports open/close/reorder/pin tabs and folder assignment; burger `expanded/collapsed/hidden` preference persists per device |
| T4.6.4 | Implement mobile top-level nav and workspace drawer behavior | E4 / S4.2 | M | Web | T4.6.1, T4.6.3 | Mobile app shows `Home/Servers/Contacts/Settings` tabs and slide-in workspace drawers per spec |

## Task Touchpoints and Validation Gates

| Task | Target touchpoints | Validation |
|---|---|---|
| T3.1.1-T3.2.1 | Friends/block/mute API handlers, DB models, fanout filters | State machine and policy integration tests pass (pending/accept/decline/block/mute paths) |
| T3.1.2, T3.1.4 | Friends list and invite share/redeem UI flows | End-to-end UI tests confirm send/respond/update and invite link/QR redeem behavior without pre-accept identity leakage |
| T3.1.3 | Contact invite token API handlers and persistence | API tests cover create/redeem and explicit error codes for invalid/expired/exhausted tokens |
| T3.1.5 | Friend-acceptance bootstrap policy path | API tests verify bootstrap material release only on accepted requests |
| T3.3.1 | Presence service + realtime event emitter | Reconnect tests meet p95 <= 1s presence propagation |
| T3.4.1 | Discovery index/query handlers + policy filter layer | Policy suite verifies blocked users excluded and rate-limit/denylist controls enforced |
| T4.0.1-T4.0.3 | Shared communication layer foundation (interface, adapters, provenance/reason taxonomy) | Contract tests confirm mode routing, provenance schema stability, and deterministic reason codes |
| T4.1.1-T4.3.1 | DM/group DM client models and server-channel message endpoints | Contract and pagination tests pass; channel CRUD/reply/mention behavior covered |
| T4.1.2 | DM policy settings and inbound permission enforcement | Policy tests confirm default friends-only and optional same-server/anyone overrides |
| T4.1.3-T4.1.10 | Infrastructure-free DM direct-connect stack (policy, pairing, diagnostics, LAN/WAN pathing, multi-device convergence) | Direct-connect conformance suite passes: no infra fallback, deterministic reason codes, pairing validation, reachability-path evidence, and profile-device convergence checks |
| T4.3.2, T4.3.3, T4.3.4, T3.3.2 | Realtime websocket event fanout, adapterization, and profile-device convergence | Ordered event stream and device-cursor hydration tests show zero loss/duplication under reconnect and late-device activation |
| T4.4.1 | Permission middleware and authorization test matrix | Bypass attempts fail across role/channel scenarios |
| T4.5.1-T4.5.4 | E2EE session bootstrap and encrypt/decrypt flows for 1:1 + group DMs | Direct user-to-user DM transport confirmed; offline retry outbox behavior and rekey/member-change tests pass |
| T4.6.1-T4.6.4 | Servers/Contacts hubs + dual-mode nav + mobile nav | UI acceptance checklist passes against `docs/product/07-ui-navigation-spec.md` |

## Entry Criteria

- Iteration 1 exit checklist is complete and OpenAPI/auth baseline is stable.
- Realtime execution must use current runtime contract authority `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`; target-state expansion planning uses `docs/contracts/realtime-events-v1.asyncapi.yaml` (resolved by `C-012`).
- Navigation implementation uses `docs/product/07-ui-navigation-spec.md` as authority.

## Exit Evidence

- Evidence pack includes policy tests, realtime ordering tests, and E2EE 1:1/group test reports.
- Navigation acceptance evidence includes desktop and mobile screenshots/checklists for `T4.6.x`.
- Delivery notes include unresolved tech debt items that affect Iteration 3 dependencies.

## UI and Flow State Mapping

| Flow | Required states (authoritative set in `docs/product/08-screen-state-spec.md`) |
|---|---|
| Contacts and friend requests | loading, search_no_results, friend_request_pending, friend_request_inbound, error |
| DM workspace onboarding | loading, empty, blocked, policy_denied, send_failed_retryable, reconnecting |
| Servers/Contacts hubs | loading, empty, search_no_results, error |

## Evidence Ledger

| Task set | Evidence artifact path | Validator |
|---|---|---|
| T3.1.x-T3.2.1 | `evidence/iteration-02/social-graph/` | policy integration test suite |
| T4.1.x-T4.5.x | `evidence/iteration-02/messaging-e2ee/` | contract + crypto integration suites |
| T4.0.1-T4.0.3, T4.3.3 | `evidence/iteration-02/networking-layer/` | communication-layer contract + adapter conformance suite |
| T4.1.3-T4.1.10 | `evidence/iteration-02/dm-connectivity/` | direct-connect conformance suite |
| T3.3.2, T4.3.4 | `evidence/iteration-02/profile-device-sync/` | profile-device convergence suite |
| T4.6.x | `evidence/iteration-02/navigation/` | UI checklist + screenshot review |

## In Progress

| ID | Task | Status | Notes |
|---|---|---|---|
| T3.1.1 | Implement friend request state machine and DB constraints | In progress | Added `POST/GET /v1/friends/requests` plus accept/decline/cancel endpoints in `api-rs` with Postgres-backed persistence, migration checksums + advisory lock, centralized auth extractor, pending-only transition guards, idempotent terminal-action semantics, auth-required revoke/session validation, atomic challenge consume-on-verify, DB-backed identity/challenge/invite durability, and DB integration tests for session/migration/restart-persistence paths |
| T3.1.2 | Build friends list UI and request actions | In progress | Contacts hub calls live friend-request endpoints with send/accept/decline actions, optimistic transition/rollback behavior, busy-state guards, and explicit screen states; HttpOnly cookie auth + CSRF header transport is enforced across contacts/session lifecycle paths |
| T3.1.3 | Implement user contact invite token create/redeem APIs | In progress | Invite APIs now run on DB-backed invite persistence path and are exposed in Contacts hub create/redeem controls; cross-service smoke validation covers auth and realtime handshake continuity |

## Done

| ID | Task | Completed In | Notes |
|---|---|---|---|
| None | - | - | - |

## Suggested Sprint Sequencing

Week 4:

- T3.1.1 -> T3.2.1
- T3.3.1 in parallel
- T3.4.1 in parallel
- T3.1.2 and T3.1.3 after request API stabilizes
- T3.1.4 and T3.1.5 after T3.1.3

Week 5:

- T4.1.1 -> T4.2.1 -> T4.2.2
- T4.0.1 -> T4.0.2 -> T4.0.3
- T4.1.2 policy defaults/overrides after T4.1.1
- T4.1.3 -> T4.1.4 -> T4.1.5
- Start T4.4.1 permission matrix design early

Week 6:

- T4.1.6 -> T4.1.7 and T4.1.8 in parallel after LAN path baseline
- T4.1.8 -> T4.1.9 -> T4.1.10 for DM multi-device convergence hardening
- T4.3.1 -> T4.3.2
- T4.3.3 after T4.3.2
- T3.3.2 -> T4.3.4 for server/presence multi-device convergence
- Finalize T4.4.1
- T4.5.1 -> T4.5.2 -> T4.5.3 -> T4.5.4
- T4.6.1, T4.6.2, T4.6.3, T4.6.4 navigation surfaces and persistence checks
- Stabilization, load tests for chat fanout, iteration demo

## Iteration 2 Exit Checklist

- Friends/block/mute/presence are functioning end-to-end.
- Direct user contact invite flow (link + QR) works end-to-end.
- In-server friend requests are mediated and do not expose raw identity material before acceptance.
- User discovery works for global and shared-server contexts.
- Shared communication layer routes both DM direct path and server-channel path through explicit adapter boundaries.
- DM direct-connect path is infrastructure-free and policy guardrails block STUN/TURN/relay fallback behavior.
- DM pairing/bootstrap works through signed out-of-band envelopes (QR/short code) with replay/expiry validation.
- DM connectivity preflight emits deterministic reason codes and guided remediation for failed direct sessions.
- DM incoming payloads converge to all profile devices (active fanout + later-active replay by cursor).
- DM/group DM and guild channels pass contract, permission, and pagination integration suites.
- Server-channel and presence events converge across all profile devices, including later-active devices.
- Permission enforcement is server-authoritative and test-covered.
- E2EE 1:1 and group DM baseline works with key exchange, rotation, and member-change rekey behavior.
- Realtime ordering and reconnect reconciliation pass load tests.
- Servers/Contacts hubs and dual-mode server navigation match `docs/product/07-ui-navigation-spec.md`.
- DM inbound policy defaults to friends-only with user override settings functioning.

## Navigation Spec Trace Matrix

| Spec requirement | Task IDs |
|---|---|
| Global `Servers Hub` with search/filter/card actions | T4.6.1 |
| Global `Contacts Hub` with search/filter/actions | T4.6.2 |
| Top-level `Home/Servers/Contacts/Settings` navigation entries | T4.6.4 |
| Sidebar + topbar tab navigation modes | T4.6.3 |
| Saved tabs/folder organization and tab indicators | T4.6.3 |
| Burger toggle state persistence (`expanded/collapsed/hidden`) | T4.6.3 |
| Mobile tabbed switcher and workspace drawer behavior | T4.6.4 |

## Execution Notes

- Keep event payload contracts versioned.
- Record authorization decision logs for node-owner debugging.
- Guild/community servers must not relay or store DM payloads.
- DM transport must not depend on infrastructure-assisted connectivity services.
- Server communication path must remain isolated from DM direct-only policy routing rules.
- Multi-device sync contracts (cursor, dedupe, hydration) must be shared across DM and server paths.
- Core owns crypto implementation tasks (`T4.5.x`); Web/API collaborate via interface contracts.
- Tag PRs and commits with task IDs (`T3.x.x`, `T4.x.x`).

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/product/08-screen-state-spec.md`
- `docs/product/09-configuration-defaults-register.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/contracts/mvp-rest-v1.openapi.yaml`
- `docs/contracts/realtime-events-v1.asyncapi.yaml`
- `docs/testing/01-mvp-verification-matrix.md`
- `docs/reference/glossary.md`
