# Scripts

Workspace automation entrypoints and reusable script implementation modules live
here. The canonical workflow and gate documentation lives in
`docs/operations/contributor-guide.md`.

## Layout

- Root `scripts/*.mjs` files are developer-facing lifecycle commands.
- `scripts/lib/` contains shared cross-platform helpers.
- `scripts/security/` contains cargo-audit policy and audit execution.
- `scripts/runtime/` contains host-process and Docker runtime managers.
- `scripts/network.mjs` is the network simulation command.
- `scripts/validators/` contains validation command entrypoints.
- `scripts/ci/` contains CI-only artifact collection.
- Top-level `fixtures/` contains shared dev seed scenarios plus runtime,
  network, and contract-parity fixture data.
- Top-level `tests/` contains test runners and test implementation.

## Commands

- `npm run setup`
- `npm run seed -- --profile dm-basic`
- `npm run reset-dev-db -- --yes`
- `npm run start`
- `npm run status`
- `npm run stop`
- `npm run network -- --profile <profile>`
- `npm run network -- --reset`
- `npm run runtime:docker -- status`
- `npm run check -- --skip-service-backed-tests`
- `npm run security`
- `npm run test -- --skip-service-backed-tests`
- `npm run test:contract-parity`
- `npm run test:runtime`
- `npm run test:network`

Validation entrypoints:

- `node scripts/validators/cargo-audit-ignore.mjs`
- `node scripts/validators/contract-parity.mjs <base> <head>`
- `node scripts/validators/dm-transport-policy.mjs`
- `node scripts/validators/docs-index-freshness.mjs <base> <head>`
- `node scripts/validators/evidence-provenance.mjs <base> <head>`
- `node scripts/validators/migration-evidence.mjs <base> <head>`
- `npm run validate:runtime-profiles`
- `npm run validate:network-profiles`

Runtime profile files live in `fixtures/runtime/profiles/`.
Network simulation profile files live in `fixtures/network/profiles/`.

The Docker runtime test stack is managed with `npm run runtime:docker`. Use it
for heavier runtime/network testing; keep normal development on host-process
`npm run start`. If the Docker runtime stack is active, use
`npm run runtime:docker -- down`; generic `npm run stop` refuses Docker runtime
state to avoid orphaning containers.
