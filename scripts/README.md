# Scripts

Workspace automation entrypoints and reusable script implementation modules live
here. The canonical workflow and gate documentation lives in
`docs/operations/contributor-guide.md`.

## Layout

- Root `scripts/*.mjs` files are developer-facing lifecycle commands.
- `scripts/lib/` contains shared cross-platform helpers for process, HTTP, JSON,
  env, path, git, and command execution code.
- `scripts/security/` contains cargo-audit policy and audit execution.
- `scripts/runtime/local/` contains host-process runtime helpers.
- `scripts/runtime/docker/` contains Docker runtime config, stack, evidence, and
  smoke assertion helpers.
- `scripts/network/` contains network simulation argument, profile, and Docker
  target helpers.
- `scripts/validators/` contains validation command entrypoints and focused
  validator implementation packages.
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

Host-process runtime services build into per-run `.local-run/targets/`
directories. This keeps Windows starts deterministic when an older `api-rs.exe`
or `realtime-rs.exe` process is still locking the normal Cargo target output;
startup avoids occupied ports and uses isolated build output instead.

Docker runtime host ports default to Alice API/realtime/web `18080`/`18081`/`3002`,
Bob API/realtime/web `18180`/`18181`/`3012`, and Toxiproxy `18474`. Override them
with the `HEXRELAY_RUNTIME_*_PORT` environment variables when a local service
already owns one of those ports; the `npm run test:runtime` and
`npm run test:network` wrappers choose available ports automatically.
