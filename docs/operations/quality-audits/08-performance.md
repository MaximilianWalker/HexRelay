# Performance Quality Audit

## Metadata

- topic_id: 08-performance
- topic: Performance
- last_audited: 2026-05-13T09:37:02Z
- source_of_truth: `docs/operations/quality-audits/08-performance.md`

## Investigation Focus

- Look for obvious algorithmic waste, unbounded queries, hot-path allocations, blocking work, and missing performance evidence for stated targets.
- Prefer measured or clearly bounded risks over speculative micro-optimization.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-08-20260513-dm-fanout-unbounded-replay-scan | P2 | confirmed | DM fanout catch-up and ack paths can scan or lock the full recipient delivery log despite page limits. | `services/api-rs/src/transport/http/handlers/dm.rs:794` loads all delivery records before applying the response `limit` in memory; `services/api-rs/src/infra/db/repos/dm_repo.rs:362` selects every `dm_fanout_delivery_log` row for an identity with no cursor or limit; `services/api-rs/src/infra/db/repos/dm_repo.rs:815` selects every row after the device cursor with `FOR UPDATE` during ack cursor advancement. | Push catch-up cursor/limit filtering into SQL and replace ack advancement with bounded contiguous-window or targeted cursor state updates; add a regression that seeds more than `MAX_PAGE_LIMIT` delivery rows and verifies bounded query behavior. | 2026-05-13T09:37:02Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T09:37:02Z | Codex | Added 1 P2 confirmed finding about unbounded DM fanout delivery-log scans in catch-up and ack hot paths. |
