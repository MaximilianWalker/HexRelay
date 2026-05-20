# Tests

Executable test harnesses and smoke checks live here.

## Layout

- `tests/run.mjs` is the canonical workspace test runner behind `npm run test`.
- `tests/contract-parity/` contains the contract-parity regression harness.
- `tests/runtime/` contains runtime and network smoke checks.

Test data does not live here. Shared deterministic repositories, scenarios, and
profiles live in top-level `fixtures/`, where they can be reused by tests,
local development commands, and CI without duplicating ownership.
