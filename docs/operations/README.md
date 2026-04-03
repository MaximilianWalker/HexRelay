# Operations Docs

## Document Metadata

- Doc ID: operations-index
- Owner: Maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-03
- Source of truth: `docs/operations/README.md`

## Quick Context

- Primary edit location for operations-document topology and process pointers.
- Detailed contributor workflow lives in `docs/operations/contributor-guide.md`.
- Latest meaningful change: 2026-04-03 added a pointer to the canonical runtime config reference so operations docs stop duplicating env inventories.

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
- Development prerequisites: `docs/operations/dev-prerequisites.md`
- Runtime config reference: `docs/reference/runtime-config-reference.md`
- Readiness corrections log: `docs/operations/readiness-corrections-log.md`
- Migration validation template: `docs/operations/migration-validation-template.md`
