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
| QA-04-20260512-contract-parity-monolith | P2 | fixed | Contract parity validation rules were concentrated in one hardcoded script, making API surface changes fragile to review. | Moved API semantic rule registries and runtime source-path constants into `scripts/contract_parity/rules.py`, moved reusable Rust query/path/request/response extraction helpers into `scripts/contract_parity/rust_parsing.py`, and kept `scripts/contract_parity/engine.py` as the orchestration layer. Temporary AST harness failed before the split and now reports `contract parity engine rule/helper split verified`; Git for Windows Bash `scripts/test-contract-parity.sh` passed. | 2026-05-18T23:27:49Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T23:27:49Z | Codex | Fixed `QA-04-20260512-contract-parity-monolith` by extracting contract parity rule registries, runtime path constants, and reusable Rust parser helpers from the engine while preserving fixture behavior. |
| 2026-05-12T21:30:05Z | Codex | Added 2 P2 found findings about split DM policy ownership and contract parity rule concentration. |
