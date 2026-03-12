# Architecture Docs

## Document Metadata

- Doc ID: architecture-index
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-12
- Source of truth: `docs/architecture/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-12 added communication networking-layer architecture plan covering shared DM/server abstraction and divergence boundaries.

## Purpose

- Use this directory for architecture decision records (ADRs).
- Keep decisions concise: context, decision, consequences, alternatives.
- Link accepted decisions back to `docs/product/01-mvp-plan.md` and `docs/product/02-prd-v1.md` when scope is affected.

## ADR Index

- `docs/architecture/adr-0001-stack-baseline.md`: MVP stack baseline (accepted).
- `docs/architecture/adr-0002-runtime-deployment-modes.md`: Runtime/deployment mode baseline (accepted).
- `docs/architecture/adr-0003-rust-service-module-architecture.md`: Rust service module architecture contract baseline (accepted).

## Architecture Specs

- `docs/architecture/02-data-lifecycle-retention-replication.md`: persistence boundaries, retention, and reconciliation behavior.
- `docs/architecture/03-rust-service-migration-baseline.md`: migration baseline, coverage snapshot, and current-to-target module mapping for Rust service structure migration.
- `docs/architecture/04-communication-networking-layer-plan.md`: shared communication layer architecture and DM-vs-server networking implementation plan.
