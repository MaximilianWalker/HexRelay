# Scripts

Workspace automation entrypoints and reusable script implementation modules live
here. The canonical workflow and gate documentation lives in
`docs/operations/contributor-guide.md`.

Use the contributor guide as the source of truth for:

- local validation commands
- PR gate expectations
- smoke/bootstrap prerequisites
- delivery and release workflow

## Layout

- Root `scripts/*.mjs`, `scripts/*.ps1`, and `scripts/*.sh` files are stable
  developer-facing command shims.
- `scripts/runtime/` contains shared host-process and Docker runtime managers.
- `scripts/network/` contains network simulation commands and profiles.
- `scripts/validators/` contains reusable validation implementation.
- `scripts/fixtures/scenarios/` contains seedable local development scenarios.
- Top-level `tests/` contains test runners and test fixtures.

Local runtime lifecycle logic is centralized in `scripts/runtime/local.mjs`.
`scripts/run.mjs`, `scripts/status.mjs`, and `scripts/stop.mjs` call that shared
manager directly. The `.ps1` and `.sh` files are compatibility shims for
developers who want native PowerShell or Bash commands.

Common script entrypoints include:

- `scripts/setup.*`
- `scripts/seed.*`
- `scripts/reset-dev-db.*`
- `scripts/run.*`
- `scripts/status.*`
- `scripts/stop.*`
- `scripts/network.*`
- `scripts/validate-*.sh`

Common test entrypoints live outside this directory:

- `tests/run.*`
- `tests/runtime/runtime-smoke.mjs`
- `tests/runtime/network-smoke.mjs`
- `tests/contract-parity/run.sh`

Local runtime testing fixture and seed details live in
`docs/planning/local-runtime-testing-plan.md`.

Runtime profile files live in `scripts/runtime/profiles/` and are validated with
`npm run validate:runtime-profiles`.

Network simulation profile files live in `scripts/network/profiles/` and are
validated with `npm run validate:network-profiles`.
Apply or reset network simulation state with `npm run network -- --profile <profile>`
or `npm run network -- --reset`.
Profiles can target runtime instance IDs, for example `local-server`,
`alice-server`, or `bob-server`.
Docker-backed profiles use Docker network controls, Toxiproxy profiles configure
Docker-only peer-link latency and timeout behavior, and app-fault profiles
configure dev-only realtime fault hooks.
Use `npm run network -- --reset --force` only for failed Docker runtime cleanup.

The Docker runtime test stack is managed with `npm run runtime:docker`. Use it
for heavier PH-05 runtime/network testing; keep normal development on
host-process `npm run start`. If the Docker runtime stack is active, use
`npm run runtime:docker -- down`; generic `npm run stop` refuses Docker runtime
state to avoid orphaning containers.
