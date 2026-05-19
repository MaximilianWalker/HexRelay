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
| QA-04-20260512-contract-parity-monolith | P2 | found | Contract parity validation rules are concentrated in one hardcoded script, making API surface changes fragile to review. | `(Get-Content scripts\contract_parity\engine.py | Measure-Object -Line).Lines` returned `1911`. The same script embeds route-scoped and auth/header rule registries in `scripts/contract_parity/engine.py:106-185`, duplicates tracked schema sets in nested extractors at `scripts/contract_parity/engine.py:667-709`, and hardcodes runtime source paths at `scripts/contract_parity/engine.py:1455-1459`. | Extract shared rule registries and parser helpers before expanding contract parity coverage for additional REST or realtime surfaces. | 2026-05-12T21:30:05Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-04-20260512-dm-policy-in-transport-handler | P2 | fixed | DM policy and validation ownership is now domain-owned instead of split through the HTTP handler. | Rechecked on 2026-05-19: `services/api-rs/src/transport/http/handlers/dm.rs` no longer defines the cited default policy, friendship, same-server policy decision, internal ACK validation, or profile-device secret validation helpers. Policy/default/device state behavior moved behind `services/api-rs/src/domain/dm/service.rs`; internal ACK and profile-device secret request validation moved behind `services/api-rs/src/domain/dm/validation.rs`; the handler now delegates to those domain functions. Temporary grep proof failed before the fix and passed after the move. | 2026-05-19 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-19T00:18:46Z | Codex issue remediator | Fixed `QA-04-20260512-dm-policy-in-transport-handler` by moving DM policy decisions and cited request validation behind domain-owned modules. |
| 2026-05-12T21:30:05Z | Codex | Added 2 P2 found findings about split DM policy ownership and contract parity rule concentration. |
