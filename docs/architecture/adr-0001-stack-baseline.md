# ADR-0001: MVP Stack Baseline

## Document Metadata

- Doc ID: adr-0001-stack-baseline
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- Decision status: accepted
- last_updated: 2026-03-04
- Source of truth: `docs/architecture/adr-0001-stack-baseline.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 documentation standardization pass.

## Context

- HexRelay is in a docs-first MVP phase and needs a stable technical baseline for initial implementation.
- The product requires low-latency realtime communication, strong identity/security primitives, and self-hostable operations.
- The team needs a stack that supports fast iteration for web UX while keeping core backend paths safe and performant.

## Decision

- Frontend baseline: Next.js with TypeScript.
- Backend baseline: Rust services for API/realtime/core flows.
- Data/infra baseline: PostgreSQL, Redis, S3-compatible object storage, WebRTC with coturn.
- Local developer environment baseline: Docker Compose for single-node setup.

## Rationale

- Next.js + TypeScript gives fast web delivery and contributor familiarity.
- Rust improves correctness and concurrency behavior for core communication paths.
- PostgreSQL + Redis + object storage + coturn is a practical, self-hostable baseline for MVP features.
- Docker Compose minimizes setup friction for early contributors.

## Consequences

- Positive:
  - Clear implementation path for Iteration 1.
  - Strong alignment between product scope and technical capabilities.
  - Predictable local environment for reproducible onboarding.
- Trade-offs:
  - Mixed-language monorepo complexity (TypeScript + Rust).
  - Additional CI complexity for multi-stack validation.

## Alternatives Considered

- Full TypeScript backend in MVP
  - Rejected due to weaker long-term fit for high-concurrency protocol-heavy services.
- Native client first
  - Rejected for MVP due to slower feedback cycle and reduced contributor accessibility.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/planning/iterations/01-sprint-board.md`
