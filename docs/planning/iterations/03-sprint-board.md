# HexRelay Iteration 3 Sprint Board

## Document Metadata

- Doc ID: iteration-03-sprint-board
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/planning/iterations/03-sprint-board.md`
- Board status: planned

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 added explicit iteration entry criteria and exit evidence requirements.

## Iteration Scope

Scope: Iteration 3 (Weeks 7-9) from `docs/product/01-mvp-plan.md`.

## Goals

- Ship competitive voice channels, 1:1 calls, and screen share lifecycle.
- Deliver media attachment pipeline with node-owner configurable storage policies.
- Establish local node-owner controls and logs (no centralized moderation layer).

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
| T5.1.1 | Implement voice signaling endpoints and websocket events | E5 / S5.1 | L | Realtime | T4.3.2 | Join/leave state remains consistent under reconnect tests with zero orphan sessions |
| T5.1.2 | Configure coturn and ICE/TURN credentials flow | E5 / S5.1 | L | Platform | T5.1.1 | Calls connect across NAT-restricted networks with TURN fallback |
| T5.2.1 | Implement 1:1 call session lifecycle (competitive quality targets) | E5 / S5.2 | M | Realtime | T5.1.1, T5.1.2 | Create/ring/accept/end flows meet p95 setup < 3s and steady-state jitter p95 < 30ms in staging tests |
| T5.3.1 | Implement screen share session lifecycle for calls/channels | E5 / S5.3 | L | Realtime | T5.1.1, T5.2.1 | Users can start/stop/view screen share with role-based access controls and reconnect within 5s after transient disconnect |
| T6.1.1 | Add attachment upload service with pre-signed URLs | E6 / S6.1 | L | API | T1.1.2 | Pre-signed URL expiry is enforced and retryable uploads succeed >= 95% in staging tests |
| T6.1.2 | Build attachment UI (upload progress/retry/preview) | E6 / S6.1 | M | Web | T6.1.1 | Users can upload/download files and recover failed transfers with retry success >= 95% in staging tests |
| T6.2.1 | Add node-owner configurable storage quotas and media policy knobs | E6 / S6.2 | M | API | T6.1.1 | Operators can set optional quotas without product hard cap |
| T6.3.1 | Implement local kick/ban APIs and local admin event log | E6 / S6.3 | M | API | T6.2.1 | Node owners can enforce local controls and query local logs with actor/target/reason fields |

## Task Touchpoints and Validation Gates

| Task | Target touchpoints | Validation |
|---|---|---|
| T5.1.1-T5.1.2 | Voice signaling handlers, websocket events, TURN credentials config | NAT-restricted connectivity suite passes with TURN fallback |
| T5.2.1 | Call lifecycle state machine and client signaling integration | p95 setup < 3s and jitter p95 < 30ms in staging test harness |
| T5.3.1 | Screen-share lifecycle controls and reconnect handling | Start/stop/rejoin tests pass with reconnect <= 5s |
| T6.1.1 | Attachment upload service and pre-signed URL generation | URL expiry and retry tests pass with >= 95% retry success |
| T6.1.2 | Attachment UI flow (progress/retry/preview) | UI e2e transfer suite passes for upload/download/retry scenarios |
| T6.2.1-T6.3.1 | Quota/policy APIs and local moderation event logs | Policy enforcement and audit-log field assertions pass |

## Entry Criteria

- Iteration 2 messaging/realtime baseline is complete and stable.
- TURN/NAT test environment is available before running voice quality gates.
- Signaling execution uses runtime contract `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`; target-state planning uses `docs/contracts/realtime-events-v1.asyncapi.yaml`.

## Exit Evidence

- Evidence pack includes NAT/TURN connectivity report and call setup/jitter metrics.
- Screen-share reconnection evidence and media retry reliability reports are attached.
- Local moderation audit-log schema checks are included in final verification output.

## UI and Flow State Mapping

| Flow | Required states (authoritative set in `docs/product/08-screen-state-spec.md`) |
|---|---|
| Voice and call lifecycle | connecting, connected, reconnecting, quality_degraded, ended, error |
| Screen share lifecycle | connecting, connected, reconnecting, ended, error |
| Attachment transfer flow | loading, success, retryable_failure, policy_denied |

## Evidence Ledger

| Task set | Evidence artifact path | Validator |
|---|---|---|
| T5.1.x-T5.3.1 | `evidence/iteration-03/voice/` | KPI profile run + reconnect suite |
| T6.1.x | `evidence/iteration-03/media/` | transfer and retry reliability suites |
| T6.2.1-T6.3.1 | `evidence/iteration-03/moderation/` | policy and audit-log assertions |

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
- Realtime owns call/screen-share lifecycle tasks (`T5.2.1`, `T5.3.1`); Web consumes stable signaling/state contracts.
- Tag PRs and commits with task IDs (`T5.x.x`, `T6.x.x`).

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/08-screen-state-spec.md`
- `docs/product/09-configuration-defaults-register.md`
- `docs/testing/01-mvp-verification-matrix.md`
- `docs/reference/glossary.md`
