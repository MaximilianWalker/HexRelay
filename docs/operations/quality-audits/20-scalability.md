# Scalability Quality Audit

## Metadata

- topic_id: 20-scalability
- topic: Scalability
- last_audited: 2026-05-14T21:52:43Z
- source_of_truth: `docs/operations/quality-audits/20-scalability.md`

## Investigation Focus

- Inspect bottlenecks, unbounded growth, async/background work boundaries, queueing, fanout behavior, and topology limits.
- Avoid premature distributed-system findings unless current MVP assumptions make them risky.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-20-20260514-dm-thread-list-global-scan | P1 | confirmed | DM thread listing pages cap the response but still aggregate and rank the global DM corpus before `LIMIT`. | `services/api-rs/src/infra/db/repos/dm_history_repo.rs:204-293` builds `message_stats` from all `dm_messages`, `participant_lists` from all `dm_thread_participants`, ranks the filtered set with `ROW_NUMBER()`, and only then applies `LIMIT $3`; `docs/contracts/runtime-rest.openapi.yaml:2201-2216` presents cursor and limit as SQL-evaluated paging for `/dm/threads`. | Replace global aggregate/rank pagination with identity-scoped keyset pagination backed by per-thread last-message summary data or an indexed materialized summary. | 2026-05-14T21:52:43Z |
| QA-20-20260514-dm-fanout-replay-unbounded | P1 | confirmed | DM fanout catch-up and ack paths scan retained delivery metadata instead of bounding work to the requested cursor/page. | `services/api-rs/src/domain/dm/validation.rs:113-149` caps catch-up `limit` at 100, but `services/api-rs/src/transport/http/handlers/dm.rs:794-849` calls `list_dm_fanout_delivery_records` and filters in memory; `services/api-rs/src/infra/db/repos/dm_repo.rs:362-376` selects every row for the recipient identity, and `services/api-rs/src/infra/db/repos/dm_repo.rs:815-830` locks every row above the current device cursor during ack advancement. Retention docs keep fanout metadata for 30 days in `docs/architecture/02-data-lifecycle-retention-replication.md:48-51`. | Push cursor and limit predicates into storage reads, and bound ack cursor advancement with indexed contiguous-window queries or compacted per-device cursor state. | 2026-05-14T21:52:43Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-20-20260514-hub-lists-unpaginated | P2 | fixed | Core hub/list endpoints return all matching rows and apply filters in memory, leaving contacts, servers, and friend-request history without page boundaries. | Added cursor/limit request fields and `next_cursor` responses in `services/api-rs/src/models.rs`, routed `/servers`, `/contacts`, and `/friends/requests` through shared offset pagination in `services/api-rs/src/transport/http/pagination.rs`, pushed server/contact/friend-request filters and page bounds into SQL in `services/api-rs/src/infra/db/repos/servers_repo.rs`, `services/api-rs/src/infra/db/repos/directory_repo.rs`, and `services/api-rs/src/infra/db/repos/friends_repo.rs`, updated runtime OpenAPI and contract-parity rules, and added focused API regression coverage. | 2026-05-18 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T17:08:44Z | Codex | Fixed QA-20-20260514-hub-lists-unpaginated by adding bounded cursor/limit pages for hub list surfaces and moving stable filters into SQL. |
| 2026-05-14T21:52:43Z | Codex | Added 2 P1 confirmed findings and 1 P2 confirmed finding about unbounded DM query/fanout work and unpaginated hub lists. |
