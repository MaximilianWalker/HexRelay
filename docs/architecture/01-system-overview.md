# HexRelay System Overview

## Document Metadata

- Doc ID: system-overview
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-03
- Source of truth: `docs/architecture/01-system-overview.md`

## Quick Context

- Purpose: provide one canonical runtime topology and trust-boundary overview for the current HexRelay system.
- Primary edit location: update this file when runtime topology, component responsibilities, or trust boundaries change.
- Latest meaningful change: 2026-04-03 created the first canonical whole-system overview and linked detailed authorities instead of relying on scattered summaries.

## Purpose

- Explain how the current system fits together at runtime.
- Identify which components own which responsibilities and data.
- Make current guarantees and non-guarantees explicit before readers drop into detailed docs.

## Runtime Modes

- `desktop local-first`
  - default product mode
  - user runs UI plus local API/realtime services on loopback
  - local browser access and embedded desktop window both target the same local runtime
- `dedicated server`
  - advanced optional mode
  - operator runs headless API/realtime services for remote clients
  - external ingress/TLS and deployment hardening become operator responsibilities

Detailed mode authority:
- `docs/architecture/adr-0002-runtime-deployment-modes.md`

## System Components

- `apps/web`
  - browser-facing UI layer
  - talks to API over HTTP and realtime over websocket
- `services/api-rs`
  - HTTP control plane
  - auth/session validation, invites, friends, DM metadata, server/channel persistence, policy checks
- `services/realtime-rs`
  - websocket/runtime fanout plane
  - websocket auth validation, live event fanout, replay hydration, presence and server-channel event delivery
- `Postgres`
  - durable node-authoritative relational state
  - API rate-limit counters when DB-backed enforcement is available
- `Redis`
  - ephemeral/shared runtime state for presence snapshots, replay logs, cursors, and pubsub fanout coordination
- object storage
  - durable blob/media storage when enabled by feature scope
- direct peer path
  - DM payload transport remains direct user-to-user, not server-relayed

## Topology by Mode

### Desktop Local-First

- UI, API, and realtime all run on the user machine.
- Loopback is the main trust boundary for local service exposure.
- Postgres/Redis may still run locally as supporting runtime dependencies.

### Dedicated Server

- API and realtime run as separate headless services.
- Browser clients connect remotely through operator-managed ingress.
- TLS terminates at ingress/reverse proxy, not directly inside current Rust services.

## Trust Boundaries

- `browser -> api-rs`
  - cookie or bearer auth
  - CSRF required on authenticated mutation routes when cookie auth is used
- `browser -> realtime-rs`
  - websocket upgrade requires valid session validation and allowed `Origin`
- `api-rs <-> realtime-rs`
  - internal service credentials are capability-scoped
  - watcher lookup and channel dispatch do not share one broad token
- `local loopback`
  - default trust boundary for desktop local-first mode
- `operator ingress`
  - dedicated deployments must provide TLS termination and header sanitization
- `direct peer DM path`
  - DM connectivity remains direct-only and must not silently fall back to project-operated relay infrastructure

Detailed authorities:
- `docs/contracts/runtime-rest-v1.openapi.yaml`
- `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`
- `docs/operations/01-mvp-runbook.md`
- `docs/architecture/04-communication-networking-layer-plan.md`

## Authoritative Data Ownership

- user/device-authoritative
  - DM payload path and direct connectivity state
  - local runtime state in desktop local-first mode
- node-authoritative
  - sessions, invites, friends, server memberships, server-channel messages
  - server-side authz and policy decisions
- ephemeral/shared runtime state
  - Redis-backed live cursors, presence snapshots, pubsub coordination, and replay acceleration state
- replicated but not primary truth
  - realtime replay acceleration state used for hydration convenience rather than primary message durability

Detailed authority:
- `docs/architecture/02-data-lifecycle-retention-replication.md`

## Core Runtime Flows

- `Auth/session`
  - client authenticates through `api-rs`
  - websocket upgrades in `realtime-rs` validate sessions against API
- `Presence`
  - websocket connect/disconnect edges publish through realtime presence flow
  - watcher resolution is API-backed and live delivery is realtime-driven
- `Server-channel messaging`
  - write path is API-authoritative and persisted first
  - realtime fanout happens afterward through protected internal publish routes
- `DM connectivity`
  - metadata/bootstrap comes from API control-plane flows
  - payload transport remains direct peer-to-peer
  - sender success semantics must mean durable sender-side acceptance, not merely attempted live fanout

## Current Guarantees and Non-Guarantees

- durable
  - API-persisted server/channel state in Postgres
  - session and social-graph persistence handled by API-side durable stores
- durable within the intended decentralized boundary
  - accepted DM messages should remain durable message history rather than expire by delivery status alone
  - delivery/replay guarantees should be defined in terms of durable acceptance plus bounded eventual catch-up, not instant reachability
- best-effort
  - server-channel live websocket fanout after persistence
  - realtime delivery is not currently transactional with REST success and has no durable outbox/retry guarantee
- bounded replay metadata
  - replay caches, retry state, and transient transport metadata may be compacted or expired
  - canonical messages should not be discarded just because delivery was delayed or already completed
- reachability vs presence
  - repeated failed delivery should downgrade current reachability assumptions without deleting the message
  - a recipient may be online yet temporarily unreachable, so reachability should not silently redefine canonical message durability

Current watch items and deferred caveats:
- `docs/operations/readiness-corrections-log.md`

## Detailed Authorities

- runtime modes: `docs/architecture/adr-0002-runtime-deployment-modes.md`
- stack baseline: `docs/architecture/adr-0001-stack-baseline.md`
- data ownership/retention: `docs/architecture/02-data-lifecycle-retention-replication.md`
- communication/networking boundaries: `docs/architecture/04-communication-networking-layer-plan.md`
- runtime REST contract: `docs/contracts/runtime-rest-v1.openapi.yaml`
- runtime realtime contract: `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`
- runtime config reference: `docs/reference/runtime-config-reference.md`
- operational procedures: `docs/operations/01-mvp-runbook.md`
