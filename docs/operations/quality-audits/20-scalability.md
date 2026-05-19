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
| QA-20-20260514-dm-fanout-replay-unbounded | P1 | confirmed | DM fanout catch-up and ack paths scan retained delivery metadata instead of bounding work to the requested cursor/page. | `services/api-rs/src/domain/dm/validation.rs:113-149` caps catch-up `limit` at 100, but `services/api-rs/src/transport/http/handlers/dm.rs:794-849` calls `list_dm_fanout_delivery_records` and filters in memory; `services/api-rs/src/infra/db/repos/dm_repo.rs:362-376` selects every row for the recipient identity, and `services/api-rs/src/infra/db/repos/dm_repo.rs:815-830` locks every row above the current device cursor during ack advancement. Retention docs keep fanout metadata for 30 days in `docs/architecture/02-data-lifecycle-retention-replication.md:48-51`. | Push cursor and limit predicates into storage reads, and bound ack cursor advancement with indexed contiguous-window queries or compacted per-device cursor state. | 2026-05-14T21:52:43Z |
| QA-20-20260514-hub-lists-unpaginated | P2 | confirmed | Core hub/list endpoints return all matching rows and apply filters in memory, leaving contacts, servers, and friend-request history without page boundaries. | `ServerListQuery`, `ContactListQuery`, and `FriendRequestListQuery` have filters but no cursor/limit in `services/api-rs/src/models.rs:98-104`, `services/api-rs/src/models.rs:178-184`, and `services/api-rs/src/models.rs:430-433`; handlers fetch all and retain in memory in `services/api-rs/src/transport/http/handlers/directory.rs:34-54`, `services/api-rs/src/transport/http/handlers/directory.rs:90-160`, and `services/api-rs/src/infra/db/repos/friends_repo.rs:160-181`. The contracts for `/servers`, `/contacts`, and `/friends/requests` expose no pagination parameters in `docs/contracts/runtime-rest.openapi.yaml:812-839`, `docs/contracts/runtime-rest.openapi.yaml:1458-1485`, and `docs/contracts/runtime-rest.openapi.yaml:2358-2376`. | Add consistent cursor/limit contracts and push search/filter predicates into SQL before response shaping. | 2026-05-14T21:52:43Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-20-20260514-dm-thread-list-global-scan | P1 | fixed | DM thread listing pages capped the response but still aggregated and ranked the global DM corpus before `LIMIT`. | `services/api-rs/migrations/0026_dm_thread_last_message_summary.sql` adds per-thread last-message summaries plus `dm_thread_participants_identity_last_message_idx`; `services/api-rs/src/db.rs` registers that migration in the runtime migrator and guards registry completeness; `services/api-rs/src/infra/db/repos/dm_history_repo.rs` now maintains those summaries and pages `/dm/threads` from `dm_thread_participants` with identity-scoped keyset predicates before `LIMIT`, aggregating participant IDs only for the selected page; the temporary source-shape and migration-registry harnesses failed before the fix and passed after it, and `cargo test -p api-rs list_dm_threads_query_uses_identity_keyset_without_global_rank --all-features` plus `cargo test -p api-rs --all-features` passed locally. | 2026-05-19T13:35:12Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T21:52:43Z | Codex | Added 2 P1 confirmed findings and 1 P2 confirmed finding about unbounded DM query/fanout work and unpaginated hub lists. |
