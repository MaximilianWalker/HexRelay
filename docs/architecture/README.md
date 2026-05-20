# Architecture Docs

## Document Metadata

- Doc ID: architecture-index
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/architecture/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-05-20 added the accepted server-node authority ADR that maps one user-facing server to one separately runnable node/runtime authority.

## Purpose

- Use this directory for architecture overviews, specifications, and decision records (ADRs).
- Keep decisions concise: context, decision, consequences, alternatives.
- Link accepted decisions back to `docs/product/01-mvp-plan.md` and `docs/product/02-prd.md` when scope is affected.

## Canonical Overview

- `docs/architecture/01-system-overview.md`: canonical runtime topology, trust-boundary, and whole-system component overview.

## ADR Index

- `docs/architecture/adr-0001-stack-baseline.md`: MVP stack baseline (accepted).
- `docs/architecture/adr-0002-runtime-deployment-modes.md`: Runtime/deployment mode, release target parity, desktop/server package boundary, and dedicated-server administration surface baseline (accepted).
- `docs/architecture/adr-0003-rust-service-module-architecture.md`: Rust service module architecture contract baseline (accepted).
- `docs/architecture/adr-0004-server-node-authority.md`: Server/node authority model: one user-facing server maps to one separately runnable server runtime/node (accepted).

## Architecture Specs

- `docs/architecture/02-data-lifecycle-retention-replication.md`: persistence boundaries, retention, and reconciliation behavior.
- `docs/architecture/03-rust-service-migration-baseline.md`: migration baseline, coverage snapshot, and current-to-target module mapping for Rust service structure migration.
- `docs/architecture/04-communication-networking-layer-plan.md`: shared communication layer architecture, server-node policy graph, opt-in discovery/relay model, and DM-vs-server networking implementation plan.
