# Maintainability Quality Audit

## Metadata

- topic_id: 04-maintainability
- topic: Maintainability
- last_audited: 2026-05-12T21:30:05Z
- source_of_truth: `docs/operations/quality-audits/04-maintainability.md`

## Investigation Focus

- Identify avoidable complexity, scattered policy, repeated logic, unclear ownership, or fragile change paths.
- Prioritize findings that would make routine MVP development harder or riskier.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-04-20260512-dm-policy-in-transport-handler | P2 | found | DM policy and validation ownership is split between the HTTP handler and domain modules. | `docs/architecture/adr-0003-rust-service-module-architecture.md:94-98` says handlers parse IO while domain services own business behavior. `services/api-rs/src/transport/http/handlers/dm.rs:26-28` imports domain DM validation, but the same handler also owns default DM policy, friendship checks, same-server policy decisions, internal ACK validation, and profile-device secret validation at `services/api-rs/src/transport/http/handlers/dm.rs:1165-1410`; `services/api-rs/src/domain/dm/validation.rs:20-193` already contains DM validation helpers. | Move DM policy decisions and request validation that are not pure IO parsing behind domain-owned functions before adding more DM delivery modes or internal endpoints. | 2026-05-12T21:30:05Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-04-20260512-contract-parity-api-density | P2 | resolved | API contract parity rules were too dense to review safely in one module. | Split the validator into `scripts/validators/contract_parity/api_rules.py`, `api_runtime.py`, `api_contract.py`, `api.py`, and `realtime.py`; the largest API-owned implementation module is now under 900 lines, direct and package validator execution both pass, and `npm run test:contract-parity` passes. | 2026-05-20T00:00:00Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-20T00:00:00Z | Codex | Resolved the contract-parity API density finding by splitting rule registries, runtime extraction, OpenAPI contract parsing, and coordinator logic into separate modules. |
| 2026-05-12T21:30:05Z | Codex | Added 2 P2 found findings about split DM policy ownership and contract parity rule concentration. |
