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
| QA-20-20260514-hub-lists-unpaginated | P2 | confirmed | Core hub/list endpoints return all matching rows and apply filters in memory, leaving contacts, servers, and friend-request history without page boundaries. | `ServerListQuery`, `ContactListQuery`, and `FriendRequestListQuery` have filters but no cursor/limit in `services/api-rs/src/models.rs:98-104`, `services/api-rs/src/models.rs:178-184`, and `services/api-rs/src/models.rs:430-433`; handlers fetch all and retain in memory in `services/api-rs/src/transport/http/handlers/directory.rs:34-54`, `services/api-rs/src/transport/http/handlers/directory.rs:90-160`, and `services/api-rs/src/infra/db/repos/friends_repo.rs:160-181`. The contracts for `/servers`, `/contacts`, and `/friends/requests` expose no pagination parameters in `docs/contracts/runtime-rest.openapi.yaml:812-839`, `docs/contracts/runtime-rest.openapi.yaml:1458-1485`, and `docs/contracts/runtime-rest.openapi.yaml:2358-2376`. | Add consistent cursor/limit contracts and push search/filter predicates into SQL before response shaping. | 2026-05-14T21:52:43Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-20-20260514-dm-fanout-replay-unbounded | P1 | fixed | DM fanout catch-up and ack paths scan retained delivery metadata instead of bounding work to the requested cursor/page. | `services/api-rs/src/infra/db/repos/dm_repo.rs` now exposes cursor/limit-bounded fanout delivery page reads, adds a device-filtered pending-delivery page query for catch-up, and advances ack cursors with an indexed recursive contiguous walk instead of locking every row above the current cursor. `services/api-rs/src/transport/http/handlers/dm.rs` now selects replay rows from `max(durable device cursor, requested cursor)` and applies the requested page limit in storage; `services/api-rs/src/tests/integration/dm_fanout_catch_up_tests.rs` covers request-cursor pagination. | 2026-05-17 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T21:52:43Z | Codex | Added 2 P1 confirmed findings and 1 P2 confirmed finding about unbounded DM query/fanout work and unpaginated hub lists. |
