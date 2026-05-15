# Reliability Quality Audit

## Metadata

- topic_id: 09-reliability
- topic: Reliability
- last_audited: 2026-05-14T02:32:13Z
- source_of_truth: `docs/operations/quality-audits/09-reliability.md`

## Investigation Focus

- Inspect partial failure handling, timeout/retry behavior, idempotency, startup/shutdown resilience, and persistence guarantees.
- Prioritize findings that can cause lost messages, stuck delivery, inconsistent runtime state, or fragile operations.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| _none_ | | | | | | |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-09-20260513-dm-outbound-retry-not-driven | P1 | fixed | DM outbound forwarding retries are scheduled but never driven by production runtime. | `services/api-rs/src/domain/dm/outbound_forwarding.rs` now exposes a bounded retry worker that repeatedly calls `retry_due_dm_outbound_forwards`, `services/api-rs/src/main.rs` starts that worker during API runtime startup, and `services/api-rs/src/tests/integration/dm_fanout_tests.rs` proves the production worker forwards a due failed static-peer record without direct test invocation of the retry function. | 2026-05-14 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T02:32:13Z | Codex automation | Fixed QA-09-20260513-dm-outbound-retry-not-driven by wiring the API runtime DM outbound-forward retry worker and adding a DB-backed worker regression. |
| 2026-05-13T12:37:27Z | Codex | Added 1 P1 found finding about undriven DM outbound forwarding retries. |
