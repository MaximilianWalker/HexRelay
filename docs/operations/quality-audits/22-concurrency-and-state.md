# Concurrency and State Quality Audit

## Metadata

- topic_id: 22-concurrency-and-state
- topic: Concurrency and State
- last_audited: 2026-05-15T03:55:33Z
- source_of_truth: `docs/operations/quality-audits/22-concurrency-and-state.md`

## Investigation Focus

- Look for races, shared-state hazards, ordering assumptions, transaction gaps, realtime sync drift, and conflict handling gaps.
- Prioritize findings that can produce duplicate, lost, stale, or unauthorized state.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-22-20260515-dm-catch-up-request-cursor-ignored | P1 | confirmed | DM fanout catch-up accepts a client cursor but ignores it when selecting replay rows, so paginated catch-up can repeat stale entries instead of advancing from the returned `next_cursor`. | `docs/contracts/runtime-rest.openapi.yaml:3408-3429` exposes optional `cursor` and `limit` on `DmFanoutCatchUpRequest`, and `docs/architecture/04-communication-networking-layer-plan.md:533-534` defines catch-up as a profile-device cursor request. `services/api-rs/src/domain/dm/validation.rs:113-149` parses and returns the cursor, but `services/api-rs/src/transport/http/handlers/dm.rs:768-810` only checks it against the delivery tail before setting `effective_cursor = last_cursor`; replay filtering at `services/api-rs/src/transport/http/handlers/dm.rs:816-849` therefore uses the durable ack cursor rather than the request cursor. A targeted test search found cursor validation coverage at `services/api-rs/src/tests/integration/dm_fanout_catch_up_tests.rs:1229` but no pagination test proving a supplied catch-up cursor changes the replay window. | Make the replay start cursor explicit, for example `max(durable_device_cursor, request_cursor)` if client cursors are advisory, or reject cursors older than the durable cursor; then add a limit-plus-cursor regression that fetches page 1 and page 2 without duplicating page 1. | 2026-05-15T03:55:33Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-22-20260515-realtime-cursors-advance-on-queue | P1 | fixed | Channel and presence replay cursors advanced when an event was queued to a websocket, before any server-observable write handoff. | Rechecked the cited paths and confirmed the issue was still real. `services/realtime-rs/src/state.rs` now carries optional outbound replay checkpoints, `services/realtime-rs/src/domain/channels.rs` and `services/realtime-rs/src/domain/presence.rs` attach channel/presence checkpoints without persisting them on queue acceptance, and `services/realtime-rs/src/transport/ws/handlers/gateway.rs` persists checkpoints only after the websocket writer successfully sends the frame. Regression assertions cover channel and presence queued messages carrying checkpoints, and the temporary static harness failed before the fix then passed after direct live/hydration cursor persistence was removed. `docs/architecture/04-communication-networking-layer-plan.md` and `docs/contracts/realtime-events-runtime.asyncapi.yaml` now document the websocket writer handoff checkpoint semantics. | 2026-05-18 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T02:03:25Z | Codex | Fixed `QA-22-20260515-realtime-cursors-advance-on-queue` by moving channel/presence replay checkpoint advancement behind successful websocket writer handoff and updating runtime contract docs. |
| 2026-05-15T03:55:33Z | Codex | Added 2 P1 confirmed findings about DM catch-up cursor pagination and realtime channel/presence cursor advancement. |
