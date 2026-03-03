# HexRelay Iteration 2 Sprint Board

Scope: Iteration 2 (Weeks 4-6) from `MVP_PLAN.md`.

Goals:

- Ship social graph and presence as stable primitives.
- Deliver DM/group DM and guild/channel messaging with permission enforcement.
- Ship global/shared-server user discovery and 1:1 E2EE DM baseline.
- Establish reliable realtime fanout patterns used by later voice and federation work.

Effort scale:

- S: 0.5-1 day
- M: 1-2 days
- L: 2-4 days
- XL: 4-6 days

## Backlog

| ID | Task | Epic/Story | Effort | Owner | Dependencies | Acceptance Criteria |
|---|---|---|---|---|---|---|
| T3.1.1 | Implement friend request state machine and DB constraints | E3 / S3.1 | L | API | Iteration 1 auth complete | Requests support pending/accepted/declined/cancelled with invariant tests |
| T3.1.2 | Build friends list UI and request actions | E3 / S3.1 | M | Web | T3.1.1 | Users can send/respond to requests from UI with optimistic updates |
| T3.2.1 | Implement block/mute logic and fanout filters | E3 / S3.2 | M | API | T3.1.1 | Blocked/muted users are excluded from delivery paths as defined |
| T3.3.1 | Implement presence service with Redis ephemeral state | E3 / S3.3 | L | Realtime | Iteration 1 infra | Presence transitions propagate in near-realtime and recover after reconnect |
| T3.4.1 | Implement global user discovery index and shared-server query | E3 / S3.4 | L | API | T3.1.1 | Discovery returns only permitted profiles with privacy-safe filtering |
| T4.1.1 | Implement DM/group DM schema and history pagination | E4 / S4.1 | L | API | T3.1.1 | DM threads support cursor pagination and unread markers |
| T4.2.1 | Implement guild/channel/role schema | E4 / S4.2 | L | API | T4.1.1 | Roles and channel membership constraints are enforced in DB/API |
| T4.2.2 | Build server/channel management UI | E4 / S4.2 | M | Web | T4.2.1 | Owners/admins can create channels and assign base roles |
| T4.3.1 | Implement message CRUD/reply/mention endpoints | E4 / S4.3 | XL | API | T4.1.1 | Messages support create/edit/delete/reply/mention with audit-safe events |
| T4.3.2 | Add websocket event fanout and optimistic UI | E4 / S4.3 | L | Realtime/Web | T4.3.1, T3.3.1 | Clients receive ordered events with conflict reconciliation |
| T4.4.1 | Add permission middleware and authorization tests | E4 / S4.4 | L | API | T4.2.1, T4.3.1 | Permission bypass attempts fail and are covered in tests |
| T4.5.1 | Implement E2EE DM key exchange/session bootstrap for 1:1 DMs | E4 / S4.5 | L | API/Core | T4.1.1 | Peers establish encrypted sessions with verifiable identity keys |
| T4.5.2 | Implement E2EE DM encrypt/decrypt flow with key rotation | E4 / S4.5 | XL | Core/Web | T4.5.1, T4.3.2 | DM messages are ciphertext on server and decrypt correctly on clients |

## In Progress

| ID | Task | Status | Notes |
|---|---|---|---|
| None | - | Not started | Awaiting Iteration 1 closeout |

## Done

| ID | Task | Completed In | Notes |
|---|---|---|---|
| None | - | - | - |

## Suggested Sprint Sequencing

Week 4:

- T3.1.1 -> T3.2.1
- T3.3.1 in parallel
- T3.4.1 in parallel
- T3.1.2 after request API stabilizes

Week 5:

- T4.1.1 -> T4.2.1 -> T4.2.2
- Start T4.4.1 permission matrix design early

Week 6:

- T4.3.1 -> T4.3.2
- Finalize T4.4.1
- T4.5.1 -> T4.5.2
- Stabilization, load tests for chat fanout, iteration demo

## Iteration 2 Exit Checklist

- Friends/block/mute/presence are functioning end-to-end.
- User discovery works for global and shared-server contexts.
- DM/group DM and guild channels are production-shaped for MVP.
- Permission enforcement is server-authoritative and test-covered.
- E2EE 1:1 DM baseline works with key exchange and rotation.
- Realtime ordering and reconnect reconciliation pass load tests.

## Execution Notes

- Keep event payload contracts versioned.
- Record authorization decision logs for node-owner debugging.
- Never persist plaintext DM payloads in server logs.
- Tag PRs and commits with task IDs (`T3.x.x`, `T4.x.x`).
