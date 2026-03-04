# HexRelay

Open-source, self-hostable communication platform with Discord-like UX and strong user ownership guarantees.

## Document Metadata

- Doc ID: project-readme
- Owner: HexRelay maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 documentation standardization pass.

## Project Stage

- Docs-first MVP planning

## Current Focus

- Finalize MVP scope and guardrails before implementation scaffold.
- Keep decentralization phased so core UX quality is not blocked.
- Maintain portability/export-import guarantees across identity and data flows.

## Start Here

- Product strategy: `docs/product/01-mvp-plan.md`
- Product requirements: `docs/product/02-prd-v1.md`
- Documentation index and source-of-truth map: `docs/README.md`
- Sprint execution boards index: `docs/planning/iterations/README.md`

## Getting Started

- Current repository state: planning-first. Implementation scaffold is not committed yet.
- If you want project direction and delivery scope first, read:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/planning/iterations/README.md`
- First runnable milestone (planned in Iteration 1):
  - Monorepo layout with `apps/web`, `services/api-rs`, `services/realtime-rs`, and `infra`
  - Local infra via `docker compose up`
  - One-command local bootstrap from a clean checkout
- Contributor workflow and PR expectations: `docs/operations/contributor-guide.md`
- Delivery change history and status transitions: `docs/planning/05-iteration-log.md`

## Contribution Context

- License target: AGPL-3.0
- Contribution policy: DCO sign-off required
- Project constraints and agent rules: `AGENTS.md`
