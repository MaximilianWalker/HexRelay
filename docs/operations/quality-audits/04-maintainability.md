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
| QA-04-20260512-contract-parity-monolith | P2 | found | Contract parity validation rules are concentrated in one hardcoded script, making API surface changes fragile to review. | `(Get-Content scripts\contract_parity\engine.py | Measure-Object -Line).Lines` returned `1911`. The same script embeds route-scoped and auth/header rule registries in `scripts/contract_parity/engine.py:106-185`, duplicates tracked schema sets in nested extractors at `scripts/contract_parity/engine.py:667-709`, and hardcodes runtime source paths at `scripts/contract_parity/engine.py:1455-1459`. | Extract shared rule registries and parser helpers before expanding contract parity coverage for additional REST or realtime surfaces. | 2026-05-12T21:30:05Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-12T21:30:05Z | Codex | Added 2 P2 found findings about split DM policy ownership and contract parity rule concentration. |
