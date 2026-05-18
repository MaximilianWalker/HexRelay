# API Design Quality Audit

## Metadata

- topic_id: 11-api-design
- topic: API Design
- last_audited: 2026-05-13T18:39:23Z
- source_of_truth: `docs/operations/quality-audits/11-api-design.md`

## Investigation Focus

- Inspect REST, realtime, internal API, contract, and UI data surfaces for consistency and predictable semantics.
- Enforce the repo rule against speculative project-owned API/realtime versioning unless a real migration constraint exists.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-11-20260513-realtime-internal-http-contract-gap | P2 | confirmed | Realtime internal HTTP routes lack contract coverage. | `services/realtime-rs/src/app/router.rs:21` defines `/internal/channels/messages/created`, `:26` `/updated`, `:30` `/deleted`, `:34` `/internal/dm/envelopes/dispatch`, and `:38` `/internal/dev/faults`; API callers depend on the channel dispatch paths in `services/api-rs/src/domain/server_channels/realtime.rs:11` and DM dispatch path in `services/api-rs/src/domain/dm/realtime.rs:17`; `rg -n -e "/internal/channels/messages" -e "/internal/dm/envelopes/dispatch" -e "/internal/dev/faults" docs/contracts` returned no matches. | Add or designate a runtime contract/parity scope for realtime internal HTTP routes, including request/response/auth semantics, or explicitly mark them private non-contract dev/runtime seams. | 2026-05-13T18:39:23Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-11-20260513-server-guild-vocabulary-split | P2 | fixed | Server/guild vocabulary remained split across API surfaces. | Canonicalized the selected API surface on `server`: `docs/contracts/mvp-rest.openapi.yaml` now uses `/servers`, `server_id`, and `Server*` schemas; `docs/contracts/realtime-events.asyncapi.yaml` target examples use server-channel/server-message naming; `services/realtime-rs/src/transport/http/internal.rs` no longer accepts `guild_id` aliases; `services/realtime-rs/src/domain/channels.rs` publish inputs now use `server_id`; and the focused vocabulary harness passed after failing before the fix. | 2026-05-18 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T18:39:23Z | Codex | Added 2 P2 confirmed findings about realtime internal HTTP contract coverage and split server/guild API vocabulary. |
