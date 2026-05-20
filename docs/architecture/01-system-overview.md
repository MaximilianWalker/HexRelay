# HexRelay System Overview

## Document Metadata

- Doc ID: system-overview
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/architecture/01-system-overview.md`

## Quick Context

- Purpose: provide one canonical runtime topology and trust-boundary overview for the current HexRelay system.
- Primary edit location: update this file when runtime topology, component responsibilities, or trust boundaries change.
- Latest meaningful change: 2026-05-20 locked the user-facing server model to one separately runnable server runtime/node authority and implemented singleton local-server API storage.

## Purpose

- Explain how the accepted MVP system fits together at runtime, including migration caveats where implementation still trails the target architecture.
- Identify which components own which responsibilities and data.
- Make current guarantees and non-guarantees explicit before readers drop into detailed docs.

## Runtime Modes

- `desktop local-first`
  - default product mode
  - user runs UI plus at least one local API/realtime server node on loopback
  - local browser access and embedded desktop window both target the same local runtime
- `dedicated server`
  - advanced optional mode
  - operator runs headless API/realtime services for remote clients
  - authorized admins manage the node through the normal HexRelay app connected to the node endpoint
  - external ingress/TLS and deployment hardening become operator responsibilities

Detailed mode authority:
- `docs/architecture/adr-0002-runtime-deployment-modes.md`

## System Components

- `apps/web`
  - browser-facing UI layer
  - talks to API over HTTP and realtime over websocket
  - renders permission-gated user and admin surfaces when the connected node authorizes them
  - may aggregate multiple joined server nodes in app state, but does not become the authority for those servers
- `services/api-rs`
  - HTTP control plane
  - auth/session validation, invites, friends, DM encrypted-envelope metadata/storage, connected-server/channel persistence, policy checks
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
- server-node P2P DM path
  - server nodes/message nodes form a dynamic policy graph for discovery, peering, relay, and delivery
  - nodes can be local-only, LAN-only, private online, allowlisted, or public opt-in
  - nodes may act as origin, delivery, relay, or discoverable nodes depending on local policy
  - nodes store/forward E2EE DM envelopes plus minimal delivery metadata only
  - never stores DM plaintext or client private keys

## Topology by Mode

### Desktop Local-First

- UI, API, and realtime all run on the user machine.
- Loopback is the main trust boundary for local service exposure.
- Postgres/Redis may still run locally as supporting runtime dependencies.
- Each local server is still a distinct node authority with its own node identity and state boundary.
- The desktop app may supervise multiple local server runtimes for convenience, but it must treat them as separate nodes rather than many servers inside one app-owned database.

### Dedicated Server

- API and realtime run as separate headless services.
- Browser clients connect remotely through operator-managed ingress.
- Each dedicated server deployment owns one node identity and one server-authoritative data boundary.
- TLS terminates at ingress/reverse proxy, not directly inside current Rust services.
- The dedicated server artifact does not ship a separate standalone admin UI by default; node owners/admins use the normal HexRelay app to connect to local, LAN, private online, or public nodes.
- Admin/operator capabilities are exposed only through authenticated API surfaces and node permissions; discoverability or LAN placement must not grant management access by itself.
- Dedicated server runtimes may participate as peers in the server-node P2P network; clients still attach to nodes rather than forming DM transport paths between recipient devices.
- A dedicated server may be hosted online and still remain private, non-discoverable, non-relaying, or invite-only.
- P2P participation is policy-scoped. Discovery, peering, relay, delivery, and durable encrypted storage are separate permissions.

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
  - remote admin access uses the same authenticated app-to-node boundary and must be explicitly permission-gated
- `server-node P2P DM path`
  - server nodes may authorize, store, and fan out ciphertext envelopes plus minimal delivery metadata only
  - server must not decrypt DM content, receive private keys, or provide an unencrypted DM mailbox/relay
  - discovery must expose only signed descriptors allowed by the discovered node's current policy
  - relay paths are valid only when every hop explicitly allows relay
  - user-consented introductions can create candidate peers only when the introduced node descriptor permits that sharing

Detailed authorities:
- `docs/contracts/runtime-rest.openapi.yaml`
- `docs/contracts/realtime-events-runtime.asyncapi.yaml`
- `docs/operations/01-mvp-runbook.md`
- `docs/architecture/04-communication-networking-layer-plan.md`

## Authoritative Data Ownership

- user/device-authoritative
  - DM plaintext, decrypted views, private keys, and local client encryption state
  - local runtime state in desktop local-first mode
- node-authoritative
  - sessions, invites, friends, server memberships, server-channel messages for that node/server authority
  - encrypted DM envelopes and minimal delivery metadata accepted by a server node/message node in the server-node P2P network
  - server-side authz and policy decisions
  - node descriptors, discovery policy, peering policy, relay policy, and delivery policy for that node
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
  - API requests are scoped to the connected node/server identity; another server id belongs behind another node endpoint
  - realtime fanout happens afterward through protected internal publish routes
- `DM delivery`
  - relationship, policy, and public bootstrap material come from API control-plane flows
  - client encrypts DM payloads before server-node delivery; message nodes in the server-node P2P network store/fan out ciphertext envelopes only
  - sender and recipient identities are portable and are not assumed to belong to a permanent primary server
  - origin, delivery, relay, and discoverable node roles are selected by current node policy and route availability
  - sender success semantics must mean durable encrypted-envelope acceptance, not merely attempted live fanout

## Current Guarantees and Non-Guarantees

- durable
  - API-persisted server/channel state in Postgres
  - session and social-graph persistence handled by API-side durable stores
- durable within the intended decentralized boundary
  - accepted encrypted DM envelopes should remain durable message history rather than expire by delivery status alone
  - delivery/replay guarantees should be defined in terms of durable acceptance plus bounded eventual catch-up, not instant reachability
- best-effort
  - server-channel live websocket fanout after persistence
  - realtime delivery is not currently transactional with REST success and has no durable outbox/retry guarantee
- bounded replay metadata
  - replay caches, retry state, and transient transport metadata may be compacted or expired
  - canonical messages should not be discarded just because delivery was delayed or already completed
- reachability vs presence
  - repeated failed delivery should downgrade current reachability assumptions without deleting the message
  - live node fanout may fail while durable encrypted-envelope delivery remains healthy, so reachability should not silently redefine canonical message durability
- server-node network topology
  - no node needs a global network view
  - small private P2P networks are valid first-class deployments
  - discovery is opt-in and does not imply peering, relay, or delivery permission

Current watch items and deferred caveats:
- `docs/operations/readiness-corrections-log.md`
- The API database now uses singleton local-server storage (`local_server` plus node-local membership/channel/role/message tables). One API runtime is one server/node authority; multi-server app views must aggregate distinct node endpoints. See `docs/architecture/adr-0004-server-node-authority.md`.

## Detailed Authorities

- runtime modes: `docs/architecture/adr-0002-runtime-deployment-modes.md`
- server/node authority: `docs/architecture/adr-0004-server-node-authority.md`
- stack baseline: `docs/architecture/adr-0001-stack-baseline.md`
- data ownership/retention: `docs/architecture/02-data-lifecycle-retention-replication.md`
- communication/networking boundaries: `docs/architecture/04-communication-networking-layer-plan.md`
- runtime REST contract: `docs/contracts/runtime-rest.openapi.yaml`
- runtime realtime contract: `docs/contracts/realtime-events-runtime.asyncapi.yaml`
- runtime config reference: `docs/reference/runtime-config-reference.md`
- operational procedures: `docs/operations/01-mvp-runbook.md`
