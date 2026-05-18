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
| QA-22-20260515-realtime-cursors-advance-on-queue | P1 | confirmed | Channel and presence replay cursors advance when an event is queued to a websocket, before any client-level receipt proves the device actually processed it. | The convergence contract requires profile devices to converge after reconnect and says durable checkpoints advance after contiguous device acks in `docs/architecture/04-communication-networking-layer-plan.md:462-466`, `:504-506`, and `:532-534`. Channel hydration skips rows at or below the stored cursor in `services/realtime-rs/src/domain/channels.rs:377-384`, while live dispatch records delivered device cursors immediately after `try_send` succeeds at `services/realtime-rs/src/domain/channels.rs:875-886` and persists them at `services/realtime-rs/src/domain/channels.rs:942-950`. Presence has the same pattern in `services/realtime-rs/src/domain/presence.rs:300-316` and `services/realtime-rs/src/domain/presence.rs:647-708`. A targeted ack search found only `dm.envelope.ack` handling and no `channel.message.*` or `presence.updated` ack surface. | Add ack-backed or otherwise server-observable delivery checkpoints for channel and presence replay, or document and test the weaker queue-accepted semantics so reconnect cannot silently skip queued-but-unprocessed events. | 2026-05-15T03:55:33Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-22-20260515-dm-catch-up-request-cursor-ignored | P1 | fixed | DM fanout catch-up accepts a client cursor but ignores it when selecting replay rows, so paginated catch-up can repeat stale entries instead of advancing from the returned `next_cursor`. | `services/api-rs/src/transport/http/handlers/dm.rs` now uses the newer of the durable device ack cursor and supplied catch-up cursor as the replay floor; `services/api-rs/src/tests/integration/dm_fanout_catch_up_tests.rs` adds `limit: 1` page-1/page-2 regression coverage; `docs/contracts/runtime-rest.openapi.yaml` documents the cursor semantics. | 2026-05-18 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18 | Codex | Fixed `QA-22-20260515-dm-catch-up-request-cursor-ignored` by applying request cursors to DM fanout catch-up replay pagination and adding focused regression coverage. |
| 2026-05-15T03:55:33Z | Codex | Added 2 P1 confirmed findings about DM catch-up cursor pagination and realtime channel/presence cursor advancement. |
