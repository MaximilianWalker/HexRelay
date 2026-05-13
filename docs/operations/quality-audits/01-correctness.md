# Correctness Quality Audit

## Metadata

- topic_id: 01-correctness
- topic: Correctness
- last_audited: 2026-05-13T02:57:40Z
- source_of_truth: `docs/operations/quality-audits/01-correctness.md`

## Investigation Focus

- Verify implemented behavior matches documented requirements and contracts.
- Look for missing edge-case handling, broken invariants, incorrect state transitions, and silent data corruption risks.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| _none_ | | | | | | |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-01-20260512-channel-replay-active-only | P1 | fixed | Server-channel late-device replay was only recorded for identities that already had an active websocket device. | `services/realtime-rs/src/domain/channels.rs` now persists replay for every normalized authorized recipient before live dispatch, and `services/realtime-rs/src/tests/ws_transport_tests.rs` covers create/update/delete hydration for a recipient with no prior active websocket device. | 2026-05-13 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T02:57:40Z | Codex automation | Fixed QA-01-20260512-channel-replay-active-only by persisting server-channel replay for no-active-device recipients and adding late-device hydration regression coverage. |
| 2026-05-12T19:49:14Z | Codex | Added 1 P1 found finding about server-channel replay entries being written only for already-active recipients. |
