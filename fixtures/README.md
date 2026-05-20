# Fixtures

Shared deterministic data used by local development, manual QA, smoke tests, and
runtime tooling lives here.

## Layout

- `fixtures/dev-seed/scenarios/` contains seedable local development scenarios
  consumed by `npm run seed` and `npm run reset-dev-db -- --profile <name>`.
- `fixtures/runtime/profiles/` contains host-process runtime topology profiles
  consumed by `npm run start -- --runtime-profile <name>`.
- `fixtures/network/profiles/` contains network simulation profiles consumed by
  `npm run network -- --profile <name>`.

Test-private fixtures stay beside their owning test harness under `tests/`, for
example `tests/contract-parity/fixtures/`.
