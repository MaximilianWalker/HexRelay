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
| QA-11-20260513-server-vocabulary-split | P2 | confirmed | Server vocabulary remains split across API surfaces. | Runtime REST and web clients use `/servers` plus `server_id` (`services/api-rs/src/app/router.rs:67`, `apps/web/lib/api.ts:329`), while older target REST and realtime examples previously carried non-canonical server aliases in `docs/contracts/mvp-rest.openapi.yaml` and `docs/contracts/realtime-events.asyncapi.yaml`. | Keep `server` as the canonical API vocabulary before new consumers adopt the target-state contracts; remove any remaining alias examples when found. | 2026-05-13T18:39:23Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T18:39:23Z | Codex | Added 2 P2 confirmed findings about realtime internal HTTP contract coverage and split server API vocabulary. |
