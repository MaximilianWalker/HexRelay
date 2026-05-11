# Operations Docs

## Document Metadata

- Doc ID: operations-index
- Owner: Maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/operations/README.md`

## Quick Context

- Primary edit location for operations-document topology and process pointers.
- Detailed contributor workflow lives in `docs/operations/contributor-guide.md`.
- Latest meaningful change: 2026-05-11 added the private mesh bootstrap operator guide, app-mediated dedicated-server administration boundary, node capability contract, and two-node HTTP forwarding smoke pointer.

## Purpose

- Use this directory for contributor/process operations docs for active implementation and release hygiene.
- Keep process lightweight and MVP-velocity oriented.
- Avoid introducing centralized lock-in assumptions in operational guidance.

## Current Operational Artifacts

- Issue templates: `.github/ISSUE_TEMPLATE/`
- Pull request template: `.github/pull_request_template.md`
- Contributor workflow: `docs/operations/contributor-guide.md`
- MVP runbook: `docs/operations/01-mvp-runbook.md`
- Dedicated server deployment: `docs/operations/02-dedicated-server-deployment.md`
- Release packaging: `docs/operations/03-release-packaging.md`
- Development prerequisites: `docs/operations/dev-prerequisites.md`
- Local runtime testing quickstart: `docs/operations/local-runtime-testing-quickstart.md`
- Private mesh bootstrap guide: `docs/operations/private-mesh-bootstrap.md`
- Runtime config reference: `docs/reference/runtime-config-reference.md`
- Local runtime testing plan: `docs/planning/local-runtime-testing-plan.md`
- Readiness corrections log: `docs/operations/readiness-corrections-log.md`
- Migration validation template: `docs/operations/migration-validation-template.md`
