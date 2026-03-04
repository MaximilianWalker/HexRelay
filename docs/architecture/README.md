# Architecture Docs

## Document Metadata

- Doc ID: architecture-index
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/architecture/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 added runtime/deployment-mode ADR for bundled desktop local-first and dedicated server operation.

## Purpose

- Use this directory for architecture decision records (ADRs).
- Keep decisions concise: context, decision, consequences, alternatives.
- Link accepted decisions back to `docs/product/01-mvp-plan.md` and `docs/product/02-prd-v1.md` when scope is affected.

## ADR Index

- `docs/architecture/adr-0001-stack-baseline.md`: MVP stack baseline (accepted).
- `docs/architecture/adr-0002-runtime-deployment-modes.md`: Runtime/deployment mode baseline (accepted).

## Architecture Specs

- `docs/architecture/02-data-lifecycle-retention-replication.md`: persistence boundaries, retention, and reconciliation behavior.
