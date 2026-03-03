# HexRelay Iteration 3 Sprint Board

Scope: Iteration 3 (Weeks 7-9) from `MVP_PLAN.md`.

Goals:

- Ship competitive voice channels, 1:1 calls, and screen share lifecycle.
- Deliver media attachment pipeline with node-owner configurable storage policies.
- Establish local node-owner controls and logs (no centralized moderation layer).

Effort scale:

- S: 0.5-1 day
- M: 1-2 days
- L: 2-4 days
- XL: 4-6 days

## Backlog

| ID | Task | Epic/Story | Effort | Owner | Dependencies | Acceptance Criteria |
|---|---|---|---|---|---|---|
| T5.1.1 | Implement voice signaling endpoints and websocket events | E5 / S5.1 | L | Realtime/API | Iteration 2 realtime events | Join/leave state changes are consistent and recover after reconnect |
| T5.1.2 | Configure coturn and ICE/TURN credentials flow | E5 / S5.1 | L | Platform | T5.1.1 | Calls connect across NAT-restricted networks with TURN fallback |
| T5.2.1 | Implement 1:1 call session lifecycle (competitive quality targets) | E5 / S5.2 | M | Realtime/Web | T5.1.1, T5.1.2 | Create/ring/accept/end flows meet latency/jitter targets |
| T5.3.1 | Implement screen share session lifecycle for calls/channels | E5 / S5.3 | L | Realtime/Web | T5.1.1, T5.2.1 | Users can start/stop/view screen share with role-based access controls |
| T6.1.1 | Add attachment upload service with pre-signed URLs | E6 / S6.1 | L | API | Iteration 1 infra | Upload flow supports retries and secure URL expiry |
| T6.1.2 | Build attachment UI (upload progress/retry/preview) | E6 / S6.1 | M | Web | T6.1.1 | Users can upload/download files and recover failed transfers |
| T6.2.1 | Add node-owner configurable storage quotas and media policy knobs | E6 / S6.2 | M | API | T6.1.1 | Operators can set optional quotas without product hard cap |
| T6.3.1 | Implement local kick/ban APIs and local admin event log | E6 / S6.3 | M | API/Web | T6.2.1 | Node owners can enforce local controls and query local logs |

## In Progress

| ID | Task | Status | Notes |
|---|---|---|---|
| None | - | Not started | Awaiting Iteration 2 completion |

## Done

| ID | Task | Completed In | Notes |
|---|---|---|---|
| None | - | - | - |

## Suggested Sprint Sequencing

Week 7:

- T5.1.1 -> T5.1.2
- Start T6.1.1 in parallel

Week 8:

- T5.2.1
- T5.3.1
- T6.1.2
- T6.2.1

Week 9:

- T6.3.1
- Voice stability soak tests and media/local-control hardening
- Iteration demo and beta readiness checkpoint

## Iteration 3 Exit Checklist

- Voice join/leave/call flows are stable under reconnect and packet loss scenarios.
- TURN fallback works in restrictive network tests.
- Screen share sessions are stable in representative beta conditions.
- Attachment pipeline enforces policy and handles retry/recovery.
- Local node-owner controls and logs are usable for day-one community ops.

## Execution Notes

- Keep voice state transitions explicit and idempotent.
- Log node-owner actions with actor, target, and reason codes.
- Avoid storing sensitive media metadata beyond operational requirements.
- Tag PRs and commits with task IDs (`T5.x.x`, `T6.x.x`).
