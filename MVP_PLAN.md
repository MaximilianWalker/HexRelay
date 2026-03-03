# HexRelay MVP Plan

HexRelay is an open-source, Discord-like communication platform built for user control: free core features, self-hostable nodes, and a phased path to decentralized discovery and federation.

## 1) Product Intent and Constraints

- Free core forever: friends, DMs, servers/channels, voice, file sharing.
- Open source first: no lock-in to a central hosted platform.
- Hybrid operation: each node can run locally or on a VPS; federation/discovery evolves in phases.
- Fast, modern UX: responsive web client first, native wrappers later.
- Rust-first backend: performance, safety, and long-term maintainability.

## 1.1) Locked Product Decisions (Founder Input)

- Target audience for beta: broad communities (gamers, dev groups, private communities).
- Multiple personas/accounts per device are supported in MVP.
- Profile model: global profile by default, optional per-server overrides.
- User discovery: direct user discovery supported globally and from shared servers.
- No cross-server DM threads between servers.
- Message defaults: edits and mentions are required; retention default is forever and configurable per server.
- Moderation model: no centralized platform moderation; only node-owner controls.
- Privacy baseline: encrypted transport and at-rest encryption everywhere; E2EE DMs in MVP.
- Voice target: competitive quality; screen share included in MVP.
- File handling: no product-level hard file size cap in MVP (server operators may set local quotas).
- Migration default keeps old device active; optional explicit device revoke is available.
- Bot ecosystem deferred until post-MVP; core stability first.
- License choice: AGPL-3.0.
- Contribution policy: DCO sign-off required, no CLA for MVP.

## 2) Architecture Decision (Locked)

### Frontend

- Web app: Next.js + TypeScript.
- Why: fastest delivery for a polished UX and broad contributor adoption.

### Backend/Core

- Rust for API/realtime/federation services.
- Why: memory safety + high concurrency + strong protocol ergonomics.

### Data and Infra

- PostgreSQL: source of truth.
- Redis: presence, ephemeral state, fanout cache.
- S3-compatible storage: media blobs.
- WebRTC + coturn: voice/call media path.

## 3) MVP Functional Scope

### In Scope

- Portable identity (public/private key), profile, device sessions.
- Multiple personas/accounts on one device.
- Friends graph: add/remove/block/mute + presence.
- User discovery (global and shared-server contexts) with direct contact requests.
- DM and group DM messaging (edits, mentions, deletes, replies).
- Servers (guilds), text channels, role/permission v1.
- Voice channels, 1:1 calls, and screen share.
- Attachments upload/download (operator-configurable quotas, no global product cap).
- Node-owner controls: local kick/ban/logging tools (no global moderation authority).
- E2EE DMs (1:1 required, group DM optional if schedule permits).
- Data ownership v1: full export and import.

### Out of Scope (Post-MVP)

- End-to-end encrypted rooms by default.
- Full decentralized DHT discovery.
- Cross-server DMs.
- Multi-region active-active global architecture.
- Video conferencing parity with enterprise platforms.
- Bot/plugin ecosystem.

## 4) Identity, Auth, and Profile Portability (Locked)

### Design Principles

- No centralized identity provider.
- No required OAuth.
- Node-local membership, user-owned global identity.
- Client-side key ownership and signed profile portability.

### Invite and Join Flow

- Server owner chooses invite mode: one-time or multi-use.
- Invite token only carries expiration as scope control in MVP.
- Invite link contains node endpoint, node fingerprint, and one-time token data.
- Client verifies node fingerprint before redeeming token.
- On first join, client binds its public key to the server membership record.

### Encryption Model

- Transport: TLS for all client/server and server/server channels.
- At-rest: database and blob encryption for node-stored data.
- DMs: E2EE with forward secrecy (server stores ciphertext and delivery metadata only).
- Keys: private keys remain client-controlled and encrypted at rest on device.

### Authentication Flow

- Server sends a nonce challenge.
- Client signs nonce with private key.
- Server verifies signature against bound public key and issues session token.
- Subsequent logins use challenge-response; no centralized credentials.

### Portable Profile Capsule

- Public identity card (signed): display name, avatar pointer/hash, bio, metadata.
- Private profile capsule (encrypted client-side): preferences and private metadata.
- Servers store signed public profile and encrypted private blob replicas.
- Profile updates are versioned and signed; newest valid version wins.

### Recovery and Multi-Device

- User receives recovery phrase or device-based key backup at onboarding.
- New devices are linked using a short-lived signed device-link token/QR from a trusted device.
- Optional offline export file for identity/profile backup.

### Restore vs Full Migration Modes

- Quick restore mode (default): QR/bootstrap transfer identity and server list, then re-sync server state in background.
- Full migration mode (explicit): move identity, profile capsules, local settings/state, and optional media cache.
- QR is used for secure session bootstrap only; large data is transferred over LAN or encrypted bundle import.

### Full Migration Workflow (Locked)

- Old device creates encrypted migration bundle (`.hxb`) signed by old device key.
- Bundle contains: identity keys (encrypted), profile data, joined server list, app settings, local-only state, optional media cache.
- Transfer options: local network direct transfer (preferred) or encrypted file export/import.
- New device verifies signature, decrypts bundle, restores state, then reconciles with servers.
- Optional cutover mode revokes old device sessions after successful import.

## 5) Rust-First Technology Stack

### Core Services (Rust)

- HTTP API: `axum` + `tower`.
- Realtime gateway: WebSocket over axum/hyper.
- Database access: `sqlx`.
- Serialization: `serde`.
- Auth/JWT: challenge-response with signed nonce + `jsonwebtoken` session tokens.
- Background jobs/events: `tokio` tasks + Redis streams/pubsub.
- Observability: `tracing`, OpenTelemetry exporter.

### Frontend and Tooling

- Next.js + TypeScript + Tailwind.
- API schema contracts: OpenAPI and generated TS client.
- Monorepo: `pnpm` workspaces + Rust workspace.
- Key management: WebCrypto for keypair operations and local encrypted key storage.

### Infrastructure

- Docker Compose for local single-node deployment.
- Kubernetes manifests later (post-MVP hardening).
- CI: lint/test/build for Rust + web.

### Data Model Additions

- `identity_keys` table (user public keys, status, created_at).
- `device_keys` table (multi-device key mapping per identity/persona).
- `node_memberships` table (node-local roles/scopes tied to public keys).
- `invites` table (hashed token, mode, TTL, usage).
- `profiles_public` table (versioned signed profile card).
- `profiles_private_replicas` table (encrypted profile blob replicas).
- `dm_sessions_e2ee` table (session metadata only; no plaintext messages).

## 6) Iteration Plan (12 Weeks)

- Iteration 1 (Weeks 1-3): Foundation + Identity.
- Iteration 2 (Weeks 4-6): Social Graph + Text Chat + E2EE DM baseline.
- Iteration 3 (Weeks 7-9): Competitive Voice + Screen Share + Media Pipeline.
- Iteration 4 (Weeks 10-12): Discovery/Portability + Full Migration + Beta Hardening.

## 7) Epics (Planned First)

### Iteration 1 Epics

- E1: Platform foundations and developer workflow.
- E2: Portable identity, invite auth, and session security.

### Iteration 2 Epics

- E3: Friends graph, user discovery, and presence model.
- E4: DMs/group DMs, guild channels, permissions v1, and E2EE DM baseline.

### Iteration 3 Epics

- E5: Competitive voice channels, calls, and screen share.
- E6: Attachments/media pipeline and node-owner control baseline.

### Iteration 4 Epics

- E7: Federation-lite discovery, node portability, and profile replication.
- E8: Reliability, observability, and beta hardening.

## 8) Stories (Derived from Epics)

### E1 Stories

- S1.1: As a contributor, I can run the stack locally with one command.
- S1.2: As a maintainer, I have CI gates for lint, test, and build.
- S1.3: As an operator, I can configure node settings via environment profiles.

### E2 Stories

- S2.1: As a user, I can create/import a portable key-based identity.
- S2.2: As a server owner, I can issue secure invite links with TTL and scopes.
- S2.3: As a user, I can join a node by redeeming an invite and binding my public key.
- S2.4: As a user, I can auth with signed challenge and manage active sessions.

### E3 Stories

- S3.1: As a user, I can send/accept/decline friend requests.
- S3.2: As a user, I can block and mute other users.
- S3.3: As a user, I can see real-time presence updates.
- S3.4: As a user, I can discover users globally and from shared servers.

### E4 Stories

- S4.1: As a user, I can create and use 1:1 and group DMs.
- S4.2: As a server owner, I can create channels and assign roles.
- S4.3: As a member, I can send/edit/delete/reply/mention in messages.
- S4.4: As a server owner, permission checks are enforced server-side.
- S4.5: As a user, my 1:1 DM messages are E2EE by default.

### E5 Stories

- S5.1: As a user, I can join/leave voice channels reliably.
- S5.2: As a user, I can perform 1:1 calls with competitive quality.
- S5.3: As a user, I can start and watch screen share in voice sessions.

### E6 Stories

- S6.1: As a user, I can upload and download attachments.
- S6.2: As a node owner, I can configure optional storage quotas/policies.
- S6.3: As a node owner, I can perform local kick/ban and review local logs.

### E7 Stories

- S7.1: As an operator, I can export/import full node user data.
- S7.2: As a user, I can discover joinable nodes from a signed registry list.
- S7.3: As a node admin, I can publish node metadata for discovery.
- S7.4: As a user, my signed public profile and encrypted private profile replicate across nodes.
- S7.5: As a user, I can perform a full device migration and restore all local state with integrity checks.

### E8 Stories

- S8.1: As a maintainer, I can observe p95 latency/errors in a simple dashboard.
- S8.2: As a maintainer, I can enforce SLO alerts for auth, messaging, and voice paths.
- S8.3: As a beta tester, onboarding and basic docs are available.

## 9) Tasks (Derived from Stories)

### Iteration 1 Tasks

- T1.1.1: Create monorepo layout (`apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`).
- T1.1.2: Add Docker Compose for Postgres, Redis, object storage emulator, coturn.
- T1.1.3: Add `make`/scripts for setup, run, test.
- T1.2.1: Configure CI matrix (Rust fmt/clippy/test + web lint/typecheck/test/build).
- T1.3.1: Add env schema validation and per-environment config templates.
- T2.1.1: Implement key identity schema and key registration endpoints.
- T2.1.2: Build client key generation/import and secure local key storage flow.
- T2.2.1: Implement invite token creation/redeem APIs with mode (one-time or multi-use) and expiration.
- T2.3.1: Implement nonce challenge-signature auth flow and session revoke endpoint.
- T2.4.1: Add node fingerprint verification in join flow and auth security tests.

### Iteration 2 Tasks

- T3.1.1: Implement friend request state machine and DB constraints.
- T3.1.2: Build friends list UI and request actions.
- T3.2.1: Implement block/mute logic in API and message fanout filters.
- T3.3.1: Implement presence service with Redis-backed ephemeral state.
- T3.4.1: Implement global user discovery index and shared-server discovery query.
- T4.1.1: Implement DM/group DM models and message history pagination.
- T4.2.1: Implement guild/channel/role schema.
- T4.2.2: Build server/channel management UI.
- T4.3.1: Implement message CRUD/reply/mention endpoints.
- T4.3.2: Add websocket event fanout and optimistic UI.
- T4.4.1: Add server-side permission middleware and tests.
- T4.5.1: Implement E2EE DM key exchange/session bootstrap for 1:1 DMs.
- T4.5.2: Implement E2EE DM encrypt/decrypt flow with forward-secrecy rotation.

### Iteration 3 Tasks

- T5.1.1: Implement voice signaling endpoints and websocket events.
- T5.1.2: Configure coturn and ICE/TURN credentials flow.
- T5.2.1: Implement 1:1 call session lifecycle.
- T5.3.1: Implement screen share session lifecycle in voice calls/channels.
- T6.1.1: Add attachment upload service with pre-signed URLs.
- T6.1.2: Build attachment UI (upload progress, retry, preview).
- T6.2.1: Add node-owner configurable storage quotas and media policies.
- T6.3.1: Implement local kick/ban APIs and local admin event log.

### Iteration 4 Tasks

- T7.1.1: Build JSON export package for account/server data and media index.
- T7.1.2: Build import flow with id remapping and conflict handling.
- T7.2.1: Implement signed registry document parser and periodic fetch.
- T7.2.2: Add node discovery UI and server join flow.
- T7.3.1: Implement node metadata publishing CLI/docs.
- T7.4.1: Implement signed public profile card sync and conflict/version handling.
- T7.4.2: Implement encrypted private profile replica sync and restore flow.
- T7.5.1: Define encrypted migration bundle format (`.hxb`) and signature verification rules.
- T7.5.2: Implement full migration export (identity, profile, settings, local state, optional media cache).
- T7.5.3: Implement migration import and reconciliation flow with server state.
- T7.5.4: Implement LAN direct transfer mode and fallback encrypted file import.
- T7.5.5: Implement optional cutover action to revoke old device sessions.
- T8.1.1: Add OpenTelemetry traces/metrics and simple dashboard views.
- T8.2.1: Define and enforce SLO alerts for latency and errors.
- T8.3.1: Publish beta admin guide + user onboarding docs.

## 10) Acceptance Criteria and Definition of Done

### Story-Level DoD

- Unit/integration tests for core logic and edge cases.
- API contracts documented in OpenAPI and validated in CI.
- Permission and auth checks covered by tests.
- Observability added for each new critical path.
- Identity and invite token secrets never appear in logs.
- No plaintext DM content is persisted on servers.

### MVP Exit Criteria

- Message delivery p95 < 300ms in same-region load test.
- Voice join success rate > 98% in beta cohort.
- Crash-free session rate > 99.5%.
- Successful export/import on at least 3 sample datasets.
- Challenge-signature auth success rate > 99.5% under load tests.
- E2EE DM decrypt success rate > 99.5% for active sessions.
- Profile capsule replication and restore works across at least 2 nodes.
- Full migration restore (bundle + reconcile) succeeds for at least 3 cross-device test scenarios.
- Screen share success rate > 95% in beta cohort.

## 11) Risks and Mitigations

- Scope creep in decentralization: limit MVP to federation-lite signed registry discovery.
- Voice instability across networks: enforce TURN fallback and connection diagnostics.
- E2EE complexity risk: start with 1:1 DMs and defer group E2EE if schedule risk emerges.
- Migration complexity: keep IDs versioned and add deterministic import conflict rules.
- Invite leakage risk: short TTL, one-time tokens, hashed token storage, revoke support.
- Key loss risk: recovery phrase/device-link flow and optional encrypted backup export.
- Migration bundle exposure risk: mandatory encryption, signature validation, and short-lived transfer sessions.

## 12) Immediate Next Actions

- Start Iteration 1 with E1 and E2 only.
- Freeze API contracts before Iteration 2 UI expansion.
- Run end-of-iteration demos with pass/fail against story acceptance.
- Execute the week-by-week board in `ITERATION_1_SPRINT_BOARD.md`.
- Track Iteration 2 delivery in `ITERATION_2_SPRINT_BOARD.md`.
- Track Iteration 3 delivery in `ITERATION_3_SPRINT_BOARD.md`.
- Track Iteration 4 delivery in `ITERATION_4_SPRINT_BOARD.md`.
