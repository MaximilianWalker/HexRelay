# HexRelay

Open-source, self-hostable communication platform with Discord-like UX and strong user ownership guarantees.

## Document Metadata

- Doc ID: project-readme
- Owner: HexRelay maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-05-20 moved shared local fixture and profile JSON under top-level `fixtures/`.

## Project Stage

- MVP planning complete; Iteration 1 foundations are complete and Iteration 2 implementation is active

## Current Focus

- Implement Iteration 2 social graph, messaging, and realtime convergence work on top of the committed foundations.
- Keep decentralization phased so core UX quality is not blocked.
- Maintain portability/export-import guarantees across identity and data flows.

## Runtime Model (Locked)

- Canonical system overview: `docs/architecture/01-system-overview.md`
- Runtime mode authority: `docs/architecture/adr-0002-runtime-deployment-modes.md`

## Start Here

- Product strategy: `docs/product/01-mvp-plan.md`
- Product requirements: `docs/product/02-prd.md`
- Documentation index and source-of-truth map: `docs/README.md`
- System overview: `docs/architecture/01-system-overview.md`
- Runtime config reference: `docs/reference/runtime-config-reference.md`
- Local runtime testing quickstart: `docs/operations/local-runtime-testing-quickstart.md`
- Dedicated server deployment baseline: `docs/operations/02-dedicated-server-deployment.md`
- Release packaging and artifact model: `docs/operations/03-release-packaging.md`
- Sprint execution boards index: `docs/planning/iterations/README.md`

## Getting Started

- Current repository state: active API/realtime/web implementation with baseline infra/CI gates running.
- Before treating current runtime behavior as fully settled, review open `watch` items in `docs/operations/readiness-corrections-log.md`; current deferred gaps include recipient-targeted realtime signaling delivery, broader semantic contract validation beyond current parity checks, process-local realtime websocket abuse-control deployment sensitivity, and docs-governance/process watches.
- If you want project direction and delivery scope first, read:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/planning/iterations/README.md`
- Baseline runnable components:
  - Monorepo layout in `apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`, and `scripts`
  - Shared local fixture/profile data in `fixtures`
  - One-command bootstrap via `npm run setup` (auto-detects Windows vs Unix)
  - Local infra via `docker compose --env-file infra/.env -f infra/docker-compose.yml up -d`
- One-command local startup via `npm run start -- --runtime-profile single` after setting `API_SESSION_SIGNING_KEYS` + `API_SESSION_SIGNING_KEY_ID` in `services/api-rs/.env` (canonical env contract: `docs/reference/runtime-config-reference.md`)
  - Workspace checks via `npm run test` (for CI parity pre-PR checks use `docs/operations/contributor-guide.md`)
- Seed local fixture scenarios with `npm run seed -- --profile dm-basic`, `contacts-edge`, or `server-chat` after local Postgres is running.
- Reset and reseed the local dev DB with `npm run reset-dev-db -- --profile dm-basic --yes`; this command refuses non-local DB targets.
- Enable `API_ENABLE_DEV_TESTING=true` only in local development to expose fixture-backed testing profile/session endpoints, then use Settings -> Testing profiles in the web app to activate Alice/Bob sessions.
- Local runtime testing plan for seeded profiles, multi-instance launch, and network simulation: `docs/planning/local-runtime-testing-plan.md`
- Operational quickstart and troubleshooting for local runtime testing: `docs/operations/local-runtime-testing-quickstart.md`
- Start multiple local instances with `npm run start -- --runtime-profile dual --seed-profile dm-basic`; inspect with `npm run status`; stop tracked processes with `npm run stop -- --runtime-profile dual`.
- Validate network simulation profile definitions with `npm run validate:network-profiles`.
- Reset network simulation state with `npm run network -- --reset`; Docker-backed, Toxiproxy, and app-fault profiles can target runtime instances such as `alice-server` or `bob-server`.
- Start the Docker runtime test stack with `npm run runtime:docker -- up --seed-profile dm-basic`; apply network profiles against `alice-server`/`bob-server`; stop it with `npm run runtime:docker -- down`.
- Run the heavier Docker runtime/network smoke with `npm run test:runtime` to validate offline, partition, Toxiproxy, app-fault, and reset paths.
- Cross-platform direct wrapper commands are documented in `docs/operations/local-runtime-testing-quickstart.md` for both PowerShell and Bash paths.

### Pre-Dev Gate (Deterministic)

1. **Terminal A**: run `npm run setup` once.
2. **Terminal A**: run `npm run start -- --runtime-profile single` (keep it running; this starts local API/realtime/web processes).
3. **Terminal B**: run `npm run status` or verify API and realtime health using the URLs printed by `npm run start`.
4. **Terminal C**: open the printed web URL and continue with local testing.
5. **Terminal D**: run `npm run test`.

Expected result: `npm run start` prints a ready stack summary with API/realtime/web URLs, health checks return success on those printed URLs, and tests pass before starting feature implementation.
- Contributor workflow and PR expectations: `docs/operations/contributor-guide.md`
- Local toolchain prerequisites: `docs/operations/dev-prerequisites.md`
- Delivery change history and status transitions: `docs/planning/05-iteration-log.md`

## Contribution Context

- License target: AGPL-3.0
- Contribution policy: DCO sign-off required
- Project constraints and agent rules: `AGENTS.md`
