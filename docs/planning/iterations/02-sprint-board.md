# HexRelay Iteration 2 Sprint Board

## Document Metadata

- Doc ID: iteration-02-sprint-board
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-14
- Source of truth: `docs/planning/iterations/02-sprint-board.md`
- Board status: in_progress

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-05-14 moved `T4.6.1` through `T4.6.4` to blocked follow-ups pending explicit UX approval.

## Iteration Scope

Scope: Iteration 2 (Weeks 4-6) from `docs/product/01-mvp-plan.md`.

## Goals

- Ship social graph and presence as stable primitives.
- Deliver DM/group DM and server-channel messaging with permission enforcement.
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
| T3.1.4 | Implement contact invite share UX (link only) | E3 / S3.1 | M | Web | T3.1.3 | User can share invite link and recipient can redeem from UI with deterministic success/error states |
| T3.1.5 | Enforce mediated identity bootstrap on friend acceptance | E3 / S3.1 | M | API | T3.1.1 | Bootstrap identity material is shared only after acceptance and is blocked on pending/declined states |
| T3.2.1 | Implement block/mute logic and fanout filters | E3 / S3.2 | M | API | T3.1.1 | Blocked/muted users are excluded from delivery paths as defined |
| T3.3.1 | Implement presence service with Redis ephemeral state | E3 / S3.3 | L | Realtime | T1.1.2 | Presence transitions propagate within p95 <= 1s and recover correctly after reconnect in integration tests |
| T3.4.1 | Implement global user discovery index and shared-server query | E3 / S3.4 | L | API | T3.1.1 | Discovery returns only permitted profiles, excludes blocked users, and enforces rate-limit/denylist controls in policy tests |
| T4.0.1 | Define shared communication layer interfaces and policy engine boundary | E4 / S4.1 | M | Core | T3.3.1 | Common communication interface supports DM and `server_channel` modes with deterministic policy routing; envelope delivery extensions land under `T4.1.7` |
| T4.0.2 | Implement initial server transport adapter boundaries (`ServerClientTransport`) | E4 / S4.1 | L | Core | T4.0.1 | Existing server-client call paths route through adapter interfaces without behavior regression |
| T4.1.1 | Implement client-side DM/group DM thread model and history pagination | E4 / S4.1 | L | Core | T3.1.1, T4.0.2 | DM threads support cursor pagination and unread markers over encrypted-envelope history without server-readable plaintext |
| T4.1.2 | Implement DM privacy policy defaults and user override settings | E4 / S4.1 | M | Core | T4.1.1 | Incoming DM policy defaults to friends-only; user can opt into same-server or anyone modes |
| T4.1.3 | Enforce E2EE DM envelope policy and CI guardrails | E4 / S4.1 | M | Core | T4.1.1 | CI rejects server-readable plaintext, private-key custody, unencrypted DM mailboxing, and plaintext relay semantics while allowing encrypted-envelope store-and-forward terminology |
| T4.1.4 | Implement relationship-scoped DM bootstrap | E4 / S4.1 | L | Core/Web | T3.1.4, T4.1.3 | Accepted contact/friend relationships release identity/profile-device bootstrap material with no recipient-device reachability, QR/manual-code pairing, or endpoint-hint requirement |
| T4.1.5 | Retire server-bypassing DM preflight and deterministic troubleshooter surfaces | E4 / S4.1 | M | Core/Web | T4.1.4 | Runtime routes, web helpers, contracts, tests, and docs no longer expose DM connectivity preflight or server-bypassing troubleshooting |
| T4.1.6 | Retire DM LAN discovery fast path | E4 / S4.1 | L | Realtime/Core | T4.1.5 | Realtime and REST surfaces no longer accept or publish recipient-device LAN discovery hints for DMs |
| T4.1.7 | Implement encrypted-envelope message-server DM delivery baseline | E4 / S4.1 | XL | API/Core | T4.1.3, T4.1.4 | `EncryptedEnvelopeServerTransport` accepts/stores/fans out ciphertext envelopes plus minimal metadata through servers/message servers; server rejects plaintext/private-key inputs; recipient-device reachability is not required |
| T4.1.8 | Add DM delivery metadata minimization, retention, and abuse controls | E4 / S4.1 | L | API/Core/Security | T4.1.7 | Metadata schema excludes plaintext/private keys, retention/deletion behavior is deterministic, and rate/abuse controls operate without plaintext inspection |
| T4.1.9 | Implement DM active-device profile fanout semantics | E4 / S4.1 | M | Core/Realtime | T4.1.7 | One accepted ciphertext envelope fanouts to all currently active devices linked to recipient profile |
| T4.1.10 | Implement DM late-device catch-up and per-device cursor dedupe | E4 / S4.1 | L | Core | T4.1.8, T4.1.9 | Devices activated after first delivery replay missed ciphertext envelopes and converge deterministically |
| T4.1.11 | Retire WAN wizard, endpoint-card, and parallel-dial DM backlog | E4 / S4.1 | L | Core/Web | T4.1.7 | Runtime, web, contracts, docs, tests, and guardrails contain no DM WAN wizard, endpoint-card, or parallel-dial surfaces |
| T4.2.2 | Build server/channel management UI | E4 / S4.2 | M | Web | T4.2.1 | Owners/admins can create channels and assign base roles |
| T4.3.2 | Add websocket event fanout for server channels | E4 / S4.3 | L | Realtime | T4.3.1, T3.3.1, T4.0.2 | Clients receive strictly ordered server-channel events; reconnect tests show no lost/duplicated events |
| T4.6.5 | Approve and implement server-channel optimistic send UI | E4 / S4.2 | M | Web | T4.3.2, explicit UX approval | Users can send server-channel messages with approved optimistic pending/sent/failure states and no duplicate websocket rows |

## Task Touchpoints and Validation Gates

| Task | Target touchpoints | Validation |
|---|---|---|
| T3.1.1-T3.2.1 | Friends/block/mute API handlers, DB models, fanout filters | State machine and policy integration tests pass (pending/accept/decline/block/mute paths) |
| T3.1.2, T3.1.4 | Friends list and invite share/redeem UI flows | End-to-end UI tests confirm send/respond/update and invite link redeem behavior without pre-accept identity leakage |
| T3.1.3 | Contact invite token API handlers and persistence | API tests cover create/redeem and explicit error codes for invalid/expired/exhausted tokens |
| T3.1.5 | Friend-acceptance bootstrap policy path | API tests verify bootstrap material release only on accepted requests |
| T3.3.1 | Presence service + realtime event emitter | Reconnect tests meet p95 <= 1s presence propagation |
| T3.4.1 | Discovery index/query handlers + policy filter layer | Policy suite verifies blocked users excluded and rate-limit/denylist controls enforced |
| T4.0.1-T4.0.3 | Shared communication layer foundation (interface, adapters, provenance/reason taxonomy) | Contract tests confirm mode routing, provenance schema stability, and deterministic reason codes |
| T4.1.1-T4.3.1 | DM/group DM client models and server-channel message endpoints | Contract and pagination tests pass; channel CRUD/reply/mention behavior covered |
| T4.1.2 | DM policy settings and inbound permission enforcement | Policy tests confirm default friends-only and optional same-server/anyone overrides |
| T4.1.3-T4.1.11 | E2EE DM envelope delivery stack (policy, relationship bootstrap, server-to-server/message-server delivery, metadata, server-bypassing surface retirement, multi-device convergence) | Envelope-delivery conformance suite passes: ciphertext-only server handling, no private-key custody, metadata minimization, deterministic reason codes, server-bypassing DM negative checks, and profile-device convergence checks |
| T4.3.2, T4.3.3, T4.3.4, T3.3.2 | Realtime websocket event fanout, adapterization, and profile-device convergence | Ordered event stream and device-cursor hydration tests show zero loss/duplication under reconnect and late-device activation |
| T4.4.1 | Permission middleware and authorization test matrix | Bypass attempts fail across role/channel scenarios |
| T4.5.1-T4.5.4 | E2EE session bootstrap and encrypt/decrypt flows for 1:1 + group DMs | Client-only decrypt/key ownership confirmed; encrypted-envelope delivery, offline catch-up behavior, and rekey/member-change tests pass |
| T4.6.1-T4.6.4 | Servers/Contacts hubs + dual-mode nav + mobile nav | UI acceptance checklist passes against `docs/product/07-ui-navigation-spec.md` |

## Entry Criteria

- Iteration 1 exit checklist is complete and OpenAPI/auth baseline is stable.
- Realtime execution must use current runtime contract authority `docs/contracts/realtime-events-runtime.asyncapi.yaml`; target-state expansion planning uses `docs/contracts/realtime-events.asyncapi.yaml` (resolved by `C-012`).
- Current runtime realtime signaling authority covers authenticated sender validation plus accepted-contact recipient-targeted live websocket delivery for cross-identity offer/answer/candidate propagation; signaling delivery is not durable/offline-queued.
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
| T4.1.3-T4.1.11 | `evidence/iteration-02/dm-connectivity/` | encrypted-envelope delivery + server-bypassing DM negative conformance suite |
| T3.3.2, T4.3.4 | `evidence/iteration-02/profile-device-sync/` | profile-device convergence suite |
| T4.6.x | `evidence/iteration-02/navigation/` | UI checklist + screenshot review |

## Done

| ID | Task | Completed In | Notes |
|---|---|---|---|
| T3.1.1 | Implement friend request state machine and DB constraints | PRs #42-#48 | `POST/GET /friends/requests` plus accept/decline/cancel endpoints with Postgres-backed persistence, migration checksums + advisory lock, centralized auth extractor, pending-only transition guards, idempotent terminal-action semantics, DB integration tests |
| T3.1.2 | Build friends list UI and request actions | PRs #42-#48 | Contacts hub with send/accept/decline actions, optimistic transition/rollback, busy-state guards, explicit screen states; HttpOnly cookie auth + CSRF header transport |
| T3.1.3 | Implement user contact invite token create/redeem APIs | PRs #42-#48 | DB-backed invite persistence, Contacts hub create/redeem controls, cross-service smoke validation |
| T3.1.5 | Enforce mediated identity bootstrap on friend acceptance | PR #49 | `GET /friends/requests/:request_id/bootstrap` endpoint; bootstrap material shared only after acceptance; 5 integration tests; OpenAPI spec updated |
| T3.1.4 | Implement contact invite share UX | PR #50 | API client functions, robust link/token parsing, copy-to-clipboard, busy/error states; QR contact-invite sharing is superseded by the later QR-only-for-server-invites scope |
| T3.2.1 | Implement block/mute logic and fanout filters | PR #51 | Block/mute CRUD plus bidirectional block checks across DM fanout and friend request creation; policy tests and OpenAPI updated |
| T3.3.1 | Implement presence service with Redis ephemeral state | PRs #53-#54 | Redis-backed presence snapshot/replay authority, websocket online/offline edge publishing, reconnect hydration, cross-service watcher resolution, and Redis-backed reconnect integration coverage (`websocket_presence_updates_propagate_and_recover_after_reconnect`) |
| T3.3.2 | Add profile-device presence convergence and late-device hydration | profile-device sync closeout branch | Realtime websocket tests cover presence fanout to multiple active profile devices, late-device online hydration, missed offline rehydration, per-device cursor dedupe, and no duplicate replay on reconnect. |
| T3.4.1 | Implement global user discovery index and shared-server query | PR #52 plus follow-up parity/policy hardening | `/discovery/users` supports `global` and `shared_server` scopes, excludes blocked and denylisted users, enforces query rate limiting, and is covered by integration tests for scope normalization, denylist enforcement, and shared-server membership filtering |
| T4.0.1 | Define shared communication layer interfaces and policy engine boundary | local working tree after PR #95 | `crates/communication-core` owns the initial shared mode/profile/policy/router boundary, deterministic routing tests cover DM/server/presence modes, current server-channel and presence integrations consume shared provenance building, and envelope-delivery extension is now tracked under `T4.1.7` |
| T4.0.2 | Implement transport adapter boundaries (`ServerClientTransport`) | T4.0.2 adapter rollout branch | `communication-core` exposes server-client dispatch bootstraps; server-channel and presence dispatch route through server-client adapters; server-bypassing DM adapter scope is superseded by the server-to-server envelope baseline |
| T4.0.3 | Implement shared session provenance and reason-code taxonomy | T4.0.3 shared provenance branch | `communication-core` exposes stable snake-case mode/profile/reason-code outputs and provenance-returning dispatch helpers; DM, server-channel, and presence ServerClientTransport dispatch paths now emit shared provenance without changing runtime routes or UX behavior |
| T4.1.1 | Implement client-side DM/group DM thread model and history pagination | local working tree after PR #95 plus DM thread regression closeout | DM thread list/messages/read APIs already provide cursor pagination, unread markers, membership scoping, and group-DM history semantics without server-readable plaintext; integration coverage now explicitly asserts the returned `group_dm` thread shape and participant set |
| T4.1.2 | Implement DM privacy policy defaults and user override settings | local working tree after PR #95 plus DM policy regression closeout | DM privacy-policy APIs already default to `friends_only`, persist per-identity override settings, enforce `friends_only`/`same_server`/`anyone` across DM paths, and now explicitly assert `same_server` readback alongside the existing enforcement coverage |
| T4.1.3 | Enforce E2EE DM envelope policy and CI guardrails | dm envelope baseline pivot branch | Direct-only policy is superseded; CI guardrails now target unsafe semantics: server-readable plaintext, private-key custody, unencrypted DM mailboxing, and plaintext relay behavior |
| T4.1.4 | Implement relationship-scoped DM bootstrap | T4.1.4 pairing closeout branch, superseded by envelope-baseline pivot | Bootstrap authority is now accepted contact/friend relationship state plus identity/profile-device material only; QR/manual-code pairing and endpoint hints are retired |
| T4.1.5 | Retire server-bypassing DM preflight and deterministic troubleshooter surfaces | T4.1.5 connectivity preflight branch, superseded by envelope-baseline pivot | DM preflight and troubleshooter surfaces are retired because normal DM delivery uses server-to-server encrypted envelopes |
| T4.1.6 | Retire DM LAN discovery fast path | T4.1.6 LAN discovery fast-path branch, superseded by envelope-baseline pivot | DM LAN discovery surfaces are retired; server discovery work must remain separately scoped |
| T4.1.7 | Implement encrypted-envelope message-server DM delivery baseline | private mesh delivery smoke branch | API accepts durable ciphertext envelopes, forwards explicit static-peer destinations over signed server-to-server HTTP, and covers local two-server forwarding smoke without recipient-device reachability |
| T4.1.8 | Add DM delivery metadata minimization, retention, and abuse controls | current backend retention branch | Configurable DM dispatch/catch-up/ack/server-forward rate limits are in place; expired fanout and outbound forwarding metadata purge without deleting canonical ciphertext history |
| T4.1.9 | Implement DM active-device profile fanout semantics | DM fanout/catch-up branches plus current realtime dispatch-summary branch | Accepted ciphertext envelopes fan out to active profile devices through verified-device realtime dispatch; internal summaries now classify target-device outcomes without plaintext/private-key access |
| T4.1.10 | Implement DM late-device catch-up and per-device cursor dedupe | DM fanout/catch-up branches | Later-active devices replay missed ciphertext envelopes with per-device cursor metadata and dedupe coverage |
| T4.1.11 | Retire WAN wizard, endpoint-card, and parallel-dial DM backlog | envelope-baseline pivot plus DM transport policy guardrail | Runtime, web, contracts, tests, fixtures, and guardrails reject retired WAN wizard, endpoint-card, and parallel-dial DM surfaces |
| T4.2.1 | Implement server-channel/role schema | T4.2.1 role permission schema branch | Persisted server roles, membership-role assignments, and per-channel role permissions now enforce server/channel scoping in DB constraints and gate server-channel read/send API access while preserving member defaults for channels without configured role permissions |
| T4.3.1 | Implement server-channel message CRUD/reply/mention endpoints | T4.3.1/T4.4.1 server-channel permission hardening branch | Runtime REST server-channel message routes support list/create/edit/delete, same-channel replies, same-server mentions, pagination, tombstones, and dispatch-safe persistence behavior with contract and integration coverage |
| T4.3.2 | Add websocket event fanout for server channels | server-channel realtime fanout branches plus reconnect duplicate closeout | API-persisted create/update/delete mutations fan out to authorized active websocket members, preserve FIFO API-to-realtime dispatch order, hydrate late profile devices through channel replay cursors, and assert no duplicate create/update/delete events after reconnect. Optimistic send UI is split to `T4.6.5` and remains blocked pending explicit approval of `docs/product/08-screen-state-spec.md`. |
| T4.3.3 | Route server-channel and presence communication through `ServerClientTransport` adapter | profile-device sync closeout branch | Server-channel API dispatch and realtime presence edge dispatch both route through `communication-core` `ServerClientTransport` helpers with stable provenance logging and no DM policy leakage. |
| T4.3.4 | Implement server-channel profile-device fanout and late-device hydration | profile-device sync closeout branch | Server-channel and presence websocket tests cover active profile-device fanout, late-device hydration, reconnect cursor dedupe, denied-channel replay exclusion, and missed presence transition replay. |
| T4.4.1 | Add permission middleware and authorization tests | T4.3.1/T4.4.1 server-channel permission hardening branch | Server/channel authorization now covers unauthenticated, outsider, cross-server path, role read-denial, role send-denial, non-author edit/delete, and removed-member bypass attempts across middleware, handler, and repository tests |
| T4.5.1 | Implement E2EE DM key exchange/session bootstrap for 1:1 DMs | T4.5.1 session-bootstrap closeout branch | `communication-core` establishes one-to-one E2EE sessions with Ed25519-signed identity bootstrap material, X25519 ephemeral agreement, HKDF-SHA256-derived session keys, trusted-peer identity-key verification, and regressions for forged identity material, tampered signatures, and wrong session contexts. |
| T4.5.2 | Implement E2EE DM encrypt/decrypt flow with key rotation (1:1) | T4.5.2 rotation planning branch | `communication-core` encrypts/decrypts client-only one-to-one ciphertext envelopes, reports rotation boundaries, derives rotated one-to-one sessions from newly signed peer bootstrap material, rejects old-session decrypt for rotated traffic, rejects group-context rotation, and serializes encrypted results without plaintext. Offline catch-up evidence remains covered by the encrypted-envelope delivery/catch-up task set. |
| T4.5.3 | Implement group DM E2EE session bootstrap and membership key updates | T4.5.3 group bootstrap/rekey branch | `communication-core` creates member-scoped group session bootstraps, derives group sessions only for current participants, creates membership-change rekey plans with added/removed identity sets, rejects removed identities before deriving new sessions, and proves old group sessions cannot decrypt post-rekey traffic. |
| T4.5.4 | Implement group DM E2EE encrypt/decrypt and failure recovery paths | T4.5.4 missing-key recovery branch | `communication-core` group sessions encrypt/decrypt XChaCha20-Poly1305 ciphertext envelopes, serialize encrypted results without plaintext, reject one-to-one sessions in the group session ring, return `session_key_missing` for post-rekey envelopes before the next group key arrives, and decrypt successfully after the rekeyed member session is inserted. |

## In Progress

| ID | Task | Status | Notes |
|---|---|---|---|
| None | - | - | - |

## Blocked Follow-Ups

| ID | Task | Status | Notes |
|---|---|---|---|
| T4.6.1 | Implement `Servers Hub` UI surface from navigation spec | blocked | Runtime UI work requires explicit approval for `NAV-APP-01` in `docs/planning/navigation-implementation-plan.md`; keep plan-only until approval exists. |
| T4.6.2 | Implement `Contacts Hub` UI surface from navigation spec | blocked | Runtime UI work requires explicit approval for `NAV-APP-02` in `docs/planning/navigation-implementation-plan.md`; keep plan-only until approval exists. |
| T4.6.3 | Implement dual server navigation modes and burger persistence | blocked | Runtime UI work requires explicit approval for `NAV-APP-03` and `NAV-APP-04` in `docs/planning/navigation-implementation-plan.md`; keep plan-only until approval exists. |
| T4.6.4 | Implement mobile top-level nav and workspace drawer behavior | blocked | Runtime UI work requires explicit approval for `NAV-APP-05` and `NAV-APP-06` in `docs/planning/navigation-implementation-plan.md`; keep plan-only until approval exists. |
| T4.6.5 | Approve and implement server-channel optimistic send UI | blocked | Plan-only proposal exists in `docs/product/08-screen-state-spec.md`; implementation must wait for explicit user approval of flow, copy, controls, and behavior. |

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

- T4.1.8 to harden metadata minimization, retention, and abuse controls over the encrypted-envelope delivery path
- Maintain T4.1.9/T4.1.10 profile-device convergence evidence while retention/abuse controls are added
- Keep T4.1.11 server-bypassing DM surface retirement evidence green before any UX-facing delivery work; all UX changes require explicit user approval
- T4.3.1 -> T4.3.2
- T4.3.3 after T4.3.2
- T3.3.2 -> T4.3.4 for server/presence multi-device convergence
- Finalize T4.4.1
- After explicit approval, resume T4.6.1, T4.6.2, T4.6.3, and T4.6.4 navigation surfaces and persistence checks from the blocked follow-ups.
- Stabilization, load tests for chat fanout, iteration demo

## Iteration 2 Exit Checklist

- Friends/block/mute/presence are functioning end-to-end.
- Server-mediated contact invite flow (link only) works end-to-end.
- In-server friend requests are mediated and do not expose raw identity material before acceptance.
- User discovery works for global and shared-server contexts.
- Shared communication layer routes encrypted-envelope DM delivery and server-channel paths through explicit adapter boundaries.
- DM envelope-delivery guardrails block server-readable plaintext, private-key custody, unencrypted mailboxing, and plaintext relay behavior.
- DM bootstrap works through accepted contact/friend relationship state and releases only identity/profile-device material needed for client-side E2EE.
- Recipient-device pairing QR/manual-code, preflight, LAN discovery, endpoint-card, WAN wizard, and parallel-dial surfaces are absent from DM runtime, web, contracts, tests, and docs.
- DM incoming payloads converge to all profile devices (active fanout + later-active replay by cursor).
- DM active-device realtime fanout has backend target summaries for queued-to-verified-websocket, pending/no-connection, unverified-device-binding, saturated-queue, and stale-connection cleanup outcomes; final delivery remains ack-backed.
- Broad profile-device announcement/discovery is not a separate MVP gap; future work should only revisit optional self/profile device-state UX or authorized endpoint-card freshness if convergence UX proves insufficient.
- Broad contact/friend device awareness is also not an MVP gap; future work should only revisit contact-authorized, pull-based endpoint-card freshness if explicit UX evidence shows stale peer metadata is hurting reconnect success.
- Broad multi-device DM convergence must operate over accepted ciphertext envelopes plus minimal replay metadata under the message-server delivery design.
- Durable DM history and replay metadata must preserve client-only plaintext/private-key boundaries; any future storage expansion must keep ciphertext-only server behavior.
- Recipient-targeted realtime signaling is accepted-contact live websocket routing for call offer/answer/candidate payloads; it is separate from DM convergence, presence discovery, durable/offline queueing, and payload delivery semantics.
- DM/group DM and server channels pass contract, permission, and pagination integration suites.
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

- Keep event payload contracts explicit and synchronized with runtime behavior.
- `T4.6.1` through `T4.6.4` implementation sequencing and the approval package live in `docs/planning/navigation-implementation-plan.md`; keep those tasks in blocked follow-ups until explicit user approval of flow, copy, controls, and behavior exists.
- Record authorization decision logs for server-owner debugging.
- Servers/message servers in the server-to-server network may store/fan out only E2EE DM envelopes and minimal delivery metadata.
- DM plaintext and private keys must remain client/device-only.
- Server communication path must remain isolated from DM ciphertext-only and client-only-key policy routing rules.
- Multi-device sync contracts (cursor, dedupe, hydration) must be shared across DM and server paths.
- Core owns crypto implementation tasks (`T4.5.x`); Web/API collaborate via interface contracts.
- Tag PRs and commits with task IDs (`T3.x.x`, `T4.x.x`).

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/product/08-screen-state-spec.md`
- `docs/product/09-configuration-defaults-register.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/contracts/mvp-rest.openapi.yaml`
- `docs/contracts/realtime-events.asyncapi.yaml`
- `docs/testing/01-mvp-verification-matrix.md`
- `docs/reference/glossary.md`
