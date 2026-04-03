# HexRelay

Open-source, self-hostable communication platform with Discord-like UX and strong user ownership guarantees.

## Document Metadata

- Doc ID: project-readme
- Owner: HexRelay maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-03
- Source of truth: `README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-04-03 shortened runtime/config onboarding by pointing to the new canonical system overview and runtime config reference.

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
- Product requirements: `docs/product/02-prd-v1.md`
- Documentation index and source-of-truth map: `docs/README.md`
- System overview: `docs/architecture/01-system-overview.md`
- Runtime config reference: `docs/reference/runtime-config-reference.md`
- Sprint execution boards index: `docs/planning/iterations/README.md`

## Getting Started

- Current repository state: active API/realtime/web implementation with baseline infra/CI gates running.
- Before treating current runtime behavior as fully settled, review open `watch` items in `docs/operations/readiness-corrections-log.md`; current deferred gaps include recipient-targeted realtime signaling delivery and replay-backlog durability.
- If you want project direction and delivery scope first, read:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/planning/iterations/README.md`
- Baseline runnable components:
  - Monorepo layout in `apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`, and `scripts`
  - One-command bootstrap via `npm run setup`
  - Local infra via `docker compose --env-file infra/.env -f infra/docker-compose.yml up -d`
- One-command local startup via `npm run run` after setting `API_SESSION_SIGNING_KEYS` + `API_SESSION_SIGNING_KEY_ID` in `services/api-rs/.env` (canonical env contract: `docs/reference/runtime-config-reference.md`)
  - Workspace checks via `npm run test` (for CI parity pre-PR checks use `docs/operations/contributor-guide.md`)

### Pre-Dev Gate (Deterministic)

1. **Terminal A**: run `npm run setup` once.
2. **Terminal A**: run `npm run run` (keep it running; this starts local API/realtime/web processes).
3. **Terminal B**: verify API health with `curl -fsS "http://127.0.0.1:8080/health"`.
4. **Terminal B**: verify realtime health with `curl -fsS "http://127.0.0.1:8081/health"`.
5. **Terminal C**: run `npm run test`.

Expected result: health checks return success and tests pass before starting feature implementation.
- Contributor workflow and PR expectations: `docs/operations/contributor-guide.md`
- Local toolchain prerequisites: `docs/operations/dev-prerequisites.md`
- Delivery change history and status transitions: `docs/planning/05-iteration-log.md`

## Contribution Context

- License target: AGPL-3.0
- Contribution policy: DCO sign-off required
- Project constraints and agent rules: `AGENTS.md`
