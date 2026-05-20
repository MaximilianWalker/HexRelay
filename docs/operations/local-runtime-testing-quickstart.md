# HexRelay Local Runtime Testing Quickstart

## Document Metadata

- Doc ID: local-runtime-testing-quickstart
- Owner: Platform and QA maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/operations/local-runtime-testing-quickstart.md`

## Quick Context

- Purpose: provide the operational quickstart and troubleshooting guide for local fixture, runtime-profile, Docker runtime, and network-simulation workflows.
- Primary edit location: update this file when local runtime commands, troubleshooting steps, or adoption guidance changes.
- Latest meaningful change: 2026-05-20 moved shared local fixture and profile JSON under top-level `fixtures/`.

## Purpose

- Use this guide when you need to seed local fixture data, launch host-process runtime profiles, run Docker runtime/network smoke tests, or collect local runtime evidence.
- Keep fixture/profile design authority in `docs/planning/local-runtime-testing-plan.md`.
- Keep environment variable inventory in `docs/reference/runtime-config-reference.md`.
- Shared local fixture/profile JSON lives under top-level `fixtures/`; test-private fixtures stay under `tests/`.

## Prerequisites

- Install the tooling in `docs/operations/dev-prerequisites.md`.
- Run commands from the repository root.
- Run `npm run setup` once after checkout or dependency changes.
- Keep normal development on host-process `npm run start`; use Docker runtime only for heavier runtime/network validation.

## Clean Checkout Flow

1. Bootstrap dependencies.

```bash
npm run setup
```

2. Start the local infra subset used by host-process runtime scripts.

```bash
docker compose --env-file infra/.env -f infra/docker-compose.yml up -d postgres redis minio
```

3. Confirm local env files exist.

```bash
npm run start
```

4. Stop the first-run stack if you only needed env bootstrap.

```bash
npm run stop
```

5. Set local signing values in `services/api-rs/.env` if they are missing.

```text
API_SESSION_SIGNING_KEYS=primary:hexrelay-dev-signing-key-change-me
API_SESSION_SIGNING_KEY_ID=primary
```

6. Enable dev testing only for local fixture/session UI flows.

```text
API_ENABLE_DEV_TESTING=true
```

## Fixture Seed And Reset

- Seed a fixture scenario after local Postgres is running.

```bash
npm run seed -- --profile dm-basic --json
```

- Reset the local dev database without fixture data.

```bash
npm run reset-dev-db -- --yes
```

- Reset and reseed the local dev database when you need a known fixture baseline.

```bash
npm run reset-dev-db -- --profile dm-basic --yes
```

- Supported scenario IDs are documented in `docs/planning/local-runtime-testing-plan.md`.
- Seed/reset commands refuse unsafe production or non-local database targets.

## Host-Process Runtime Profiles

- Start one clean local app instance. This is the default startup path and does not seed data or activate fixture personas.

```bash
npm run start
```

- The shared lifecycle implementation is `scripts/runtime/local.mjs`; `npm run start`, `npm run status`, and `npm run stop` are the preferred cross-platform entrypoints.

- Start Alice/Bob side-by-side runtime instances only when explicitly testing fixture scenarios.

```bash
npm run start -- --runtime-profile dual --seed-profile dm-basic
```

- Inspect tracked processes, ports, and health.

```bash
npm run status
```

- Stop tracked host-process runtime instances.

```bash
npm run stop -- --runtime-profile dual
```

- Windows direct wrappers are available when you need PowerShell explicitly.

```powershell
.\scripts\run.ps1 -RuntimeProfile dual -SeedProfile dm-basic
.\scripts\status.ps1
.\scripts\stop.ps1 -RuntimeProfile dual
```

- Unix direct wrappers are available when you need Bash explicitly.

```bash
./scripts/run.sh --runtime-profile dual --seed-profile dm-basic
./scripts/status.sh
./scripts/stop.sh --runtime-profile dual
```

## Testing Profile UI

- Start a local web runtime with `API_ENABLE_DEV_TESTING=true` in `services/api-rs/.env`.
- Open the web URL printed by `npm run start`.
- Go to Settings -> Testing profiles.
- Activate `alice.primary` or `bob.primary` to write the matching fixture persona/session into browser storage.
- Use separate browser profiles, separate browser contexts, or separate runtime web URLs for Alice/Bob side-by-side checks.

## Docker Runtime And Network Simulation

- Start the Docker runtime test stack.

```bash
npm run runtime:docker -- up --seed-profile dm-basic
```

- Check Docker runtime health.

```bash
npm run runtime:docker -- status
```

- Apply a Docker-backed network profile.

```bash
npm run network -- --profile offline-alice
npm run network -- --profile partition-alice-bob
```

- Apply a Toxiproxy-backed profile against a runtime target.

```bash
npm run network -- --profile high-latency --target alice-server
npm run network -- --profile packet-loss --target alice-server
```

- Apply a realtime app-fault profile against a runtime target.

```bash
npm run network -- --profile flaky-mobile --target alice-server
```

- Reset network simulation state after every manual profile run.

```bash
npm run network -- --reset
```

- Stop the Docker runtime test stack.

```bash
npm run runtime:docker -- down
```

- Use forced cleanup only after a failed smoke leaves stale runtime state.

```bash
npm run runtime:docker -- down --force
```

## Validation Commands

- Validate profile definitions.

```bash
npm run validate:runtime-profiles
npm run validate:network-profiles
```

- Run full Docker runtime/network smoke.

```bash
npm run test:runtime
```

- Run network scenario smoke explicitly.

```bash
npm run test:network
```

- Run runtime-only smoke with evidence output.

```bash
node scripts/runtime/docker.mjs smoke --scope runtime --evidence-dir .local-run/evidence/runtime-smoke
```

- Run full runtime/network smoke with evidence output.

```bash
node scripts/runtime/docker.mjs smoke --scope all --evidence-dir .local-run/evidence/local-runtime-smoke
```

## Evidence Artifacts

- Local smoke evidence can be written under `.local-run/evidence/` for ad hoc runs.
- Durable release or audit evidence must follow `docs/testing/01-mvp-verification-matrix.md`.
- Durable evidence folders must include `summary.md`, `validators.txt`, `provenance.json`, and an `outputs/` directory.
- Store generated smoke artifacts under `outputs/` when promoting local smoke output to durable evidence.
- Runtime/network smoke output files are:
  - `scenario-config.json`
  - `runtime-status-before.json`
  - `runtime-status-after.json`
  - `event-log.ndjson`
  - `verdict.md`

## Troubleshooting

| Symptom | Likely cause | Recovery |
|---|---|---|
| `npm run stop` refuses to stop | Docker runtime state is active | Run `npm run runtime:docker -- down` |
| Docker runtime smoke failed during cleanup | Containers or network state remained active | Run `npm run runtime:docker -- down --force`, then `npm run runtime:docker -- status --json` |
| Network profile remains applied | `.local-run/network-state.json` still tracks a profile | Run `npm run network -- --reset`; use `--force` only after failed Docker cleanup |
| Host-process ports are busy | Another local runtime or app is already running | The shared runner picks free fallback ports; run `npm run status`, then stop tracked profiles with `npm run stop -- --runtime-profile <profile>` when the process is managed by this repo |
| Next reports another dev server | A stale or unmanaged Next process is using an old dist directory | Managed starts use per-run `.next-*` directories; stop the unmanaged process separately only if you need its exact port |
| Testing profiles are hidden or inert | `API_ENABLE_DEV_TESTING` is unset or false | Set `API_ENABLE_DEV_TESTING=true` only in local development and restart API/web |
| Seed/reset refuses the database | Env points at a non-local or production-looking DB | Fix `API_DATABASE_URL` and `API_ENVIRONMENT=development`; do not bypass this for shared data |
| Docker network profile fails on host-process profile | Docker profiles need Docker container targets | Start `npm run runtime:docker -- up --seed-profile dm-basic` and target `alice-server` or `bob-server` |
| Runtime status is stale | `.local-run/` references old processes | Run the matching stop/down command first; delete stale `.local-run/` files only after confirming no owned process/container is running |

## Safety Rules

- Keep fixture/session endpoints dev-only.
- Keep local runtime test ports bound to loopback.
- Keep seed/reset restricted to local development databases.
- Do not use network simulation to add server-bypassing DM transport or plaintext relay behavior.
- Reset network simulation state after manual failure testing.

## Related Documents

- `README.md`
- `docs/planning/local-runtime-testing-plan.md`
- `docs/reference/runtime-config-reference.md`
- `docs/testing/01-mvp-verification-matrix.md`
- `docs/operations/dev-prerequisites.md`
- `scripts/README.md`
