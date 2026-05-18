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
| QA-11-20260513-server-guild-vocabulary-split | P2 | confirmed | Server/guild vocabulary remains split across API surfaces. | Runtime REST and web clients use `/servers` plus `server_id` (`services/api-rs/src/app/router.rs:67`, `apps/web/lib/api.ts:329`), but target REST still exposes `/guilds` and `guild_id` (`docs/contracts/mvp-rest.openapi.yaml:239`, `:268`, `:287`, `:504`, `:699`, `:712`); realtime internal dispatch also accepts `guild_id` aliases and domain inputs still name `pub guild_id` (`services/realtime-rs/src/transport/http/internal.rs:28`, `:40`, `:52`; `services/realtime-rs/src/domain/channels.rs:124`, `:135`, `:146`), while target realtime examples retain `guild_channel_*` and `guild-message-service` names (`docs/contracts/realtime-events.asyncapi.yaml:422`, `:428`, `:443`, `:449`, `:464`, `:470`). | Pick one canonical API vocabulary for community containers before new consumers adopt the target-state contracts; remove or explicitly document any remaining compatibility aliases/examples. | 2026-05-13T18:39:23Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-11-20260513-realtime-internal-http-contract-gap | P2 | fixed | Realtime internal HTTP routes lacked contract coverage. | Added `docs/contracts/realtime-internal.openapi.yaml` for the realtime service-to-service channel dispatch, DM envelope dispatch, and dev-fault routes; updated `scripts/contract_parity/validator.py` and `scripts/contract_parity/engine.py` to enforce internal route inventory, required `x-hexrelay-internal-token`, JSON request body, and response-status parity; `bash scripts/test-contract-parity.sh` includes realtime internal HTTP pass/fail fixture coverage. | 2026-05-18 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T18:39:23Z | Codex | Added 2 P2 confirmed findings about realtime internal HTTP contract coverage and split server/guild API vocabulary. |
