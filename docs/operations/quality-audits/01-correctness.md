# Correctness Quality Audit

## Metadata

- topic_id: 01-correctness
- topic: Correctness
- last_audited: 2026-05-12T19:49:14Z
- source_of_truth: `docs/operations/quality-audits/01-correctness.md`

## Investigation Focus

- Verify implemented behavior matches documented requirements and contracts.
- Look for missing edge-case handling, broken invariants, incorrect state transitions, and silent data corruption risks.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-01-20260512-channel-replay-active-only | P1 | found | Server-channel late-device replay is only recorded for identities that already have an active websocket device. | `docs/product/01-mvp-plan.md:224`-`230`, `docs/product/02-prd.md:197`-`200`, and `docs/architecture/04-communication-networking-layer-plan.md:503`-`506` require server-channel and presence state to hydrate later-active devices by per-device cursor. `services/realtime-rs/src/domain/channels.rs:444`, `:555`, and `:666` persist channel replay only for `active_replay_recipients`, while `services/realtime-rs/src/domain/channels.rs:747`-`763` filters recipients to identities with an existing connection that has a device id before writing `channels:recipient_stream_log:*`. Existing late-device tests exercise replay after a primary device is connected, for example `services/realtime-rs/src/tests/ws_transport_tests.rs:1766`-`2110`, so the no-active-device case can pass unverified. | Persist channel replay for all authorized recipients or document a narrower server-channel hydration guarantee, then add a regression where a member has no active websocket during create/update/delete and a later device hydrates by cursor. | 2026-05-12 |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-12T19:49:14Z | Codex | Added 1 P1 found finding about server-channel replay entries being written only for already-active recipients. |
