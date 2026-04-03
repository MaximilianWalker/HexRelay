# Architecture Docs

## Document Metadata

- Doc ID: architecture-index
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-03
- Source of truth: `docs/architecture/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-04-03 added the canonical system overview so runtime topology and trust boundaries no longer have to be reconstructed from scattered docs.

## Purpose

- Use this directory for architecture overviews, specifications, and decision records (ADRs).
- Keep decisions concise: context, decision, consequences, alternatives.
- Link accepted decisions back to `docs/product/01-mvp-plan.md` and `docs/product/02-prd-v1.md` when scope is affected.

## Canonical Overview

- `docs/architecture/01-system-overview.md`: canonical runtime topology, trust-boundary, and whole-system component overview.

## ADR Index

- `docs/architecture/adr-0001-stack-baseline.md`: MVP stack baseline (accepted).
- `docs/architecture/adr-0002-runtime-deployment-modes.md`: Runtime/deployment mode baseline (accepted).
- `docs/architecture/adr-0003-rust-service-module-architecture.md`: Rust service module architecture contract baseline (accepted).

## Architecture Specs

- `docs/architecture/02-data-lifecycle-retention-replication.md`: persistence boundaries, retention, and reconciliation behavior.
- `docs/architecture/03-rust-service-migration-baseline.md`: migration baseline, coverage snapshot, and current-to-target module mapping for Rust service structure migration.
- `docs/architecture/04-communication-networking-layer-plan.md`: shared communication layer architecture and DM-vs-server networking implementation plan.
