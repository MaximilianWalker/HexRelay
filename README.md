# HexRelay

Open-source, self-hostable communication platform with Discord-like UX and strong user ownership guarantees.

## Document Metadata

- Doc ID: project-readme
- Owner: HexRelay maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-10
- Source of truth: `README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-10 aligned quickstart to canonical setup/test docs and standardized Rust policy to latest stable.

## Project Stage

- MVP planning complete and development bootstrap in progress

## Current Focus

- Implement Iteration 1 foundations on top of the committed scaffolds.
- Keep decentralization phased so core UX quality is not blocked.
- Maintain portability/export-import guarantees across identity and data flows.

## Runtime Model (Locked)

- Primary product mode is a downloadable desktop app that runs off-grid without a central hosted control plane.
- Desktop distribution bundles UI plus local API/realtime runtime components.
- Local install supports two UI entry options: embedded desktop window and optional local-browser access to the same local runtime.
- Users can run dedicated server deployments (headless API/realtime) as an optional advanced mode.
- Architecture stays multi-component at runtime (UI, API service, realtime service) even when distributed as one installer.

## Start Here

- Product strategy: `docs/product/01-mvp-plan.md`
- Product requirements: `docs/product/02-prd-v1.md`
- Documentation index and source-of-truth map: `docs/README.md`
- Sprint execution boards index: `docs/planning/iterations/README.md`

## Getting Started

- Current repository state: scaffolds and baseline infra/CI are initialized for development start.
- If you want project direction and delivery scope first, read:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/planning/iterations/README.md`
- Baseline runnable components:
  - Monorepo layout in `apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`, and `scripts`
  - Local infra via `docker compose --env-file infra/.env -f infra/docker-compose.yml up -d`
  - One-command bootstrap via `npm run setup`
  - One-command local startup via `npm run run` (canonical env contract: `docs/operations/dev-prerequisites.md`)
  - Workspace checks via `npm run test` (for CI parity pre-PR checks use `docs/operations/contributor-guide.md`)
- Contributor workflow and PR expectations: `docs/operations/contributor-guide.md`
- Local toolchain prerequisites: `docs/operations/dev-prerequisites.md`
- Delivery change history and status transitions: `docs/planning/05-iteration-log.md`

## Contribution Context

- License target: AGPL-3.0
- Contribution policy: DCO sign-off required
- Project constraints and agent rules: `AGENTS.md`
