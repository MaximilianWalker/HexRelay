# HexRelay MVP Plan

HexRelay is an open-source, Discord-like communication platform built for user control: free core features, self-hostable nodes, and a phased path to decentralized discovery and federation.

## Document Metadata

- Doc ID: mvp-plan
- Owner: Product and architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/product/01-mvp-plan.md`

## Quick Context

- Primary edit location for product intent, constraints, architecture baseline, and epics/stories.
- Iteration task sequencing and task-level status are canonical in `docs/planning/iterations/README.md`.
- Dependency/risk severity updates are canonical in `docs/product/04-dependencies-risks.md`.
- Latest meaningful change: 2026-03-04 locked direct peer DM transport with best-effort offline retry policy.

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
- DM transport is independent of guild/community servers (no server-mediated DM relay).
- Message defaults: edits and mentions are required; retention default is forever and configurable per server.
- Moderation model: no centralized platform moderation; only node-owner controls.
- Privacy baseline: encrypted transport and at-rest encryption everywhere; E2EE DMs in MVP.
- DM transport baseline: direct user-to-user channels; guild/community servers do not relay/store DM payloads.
- Voice target: competitive quality; screen share included in MVP.
- UI direction: heavily Discord-inspired interaction model, except server navigation uses scalable list/card paradigms (no small circular server rail).
- Server navigation supports dual mode: sidebar list/folders and topbar tabbed navigation (browser-like tabs/saved tabs).
- Server navigation chrome can be collapsed/hidden with a burger toggle while inside a server workspace.
- File handling: no product-level hard file size cap in MVP (server operators may set local quotas).
- Migration default keeps old device active; optional explicit device revoke is available.
- Bot ecosystem deferred until post-MVP; core stability first.
- License choice: AGPL-3.0.
- Contribution policy: DCO sign-off required, no CLA for MVP.

## 1.2) Decision Log

- 2026-03-03: Added docs metadata and decision-log section to standardize documentation governance.
- 2026-03-04: Locked MVP invite scope semantics to mode + expiration + max-uses only; role/channel scoped invites deferred post-MVP.
- 2026-03-04: Added MVP Crypto Profile v1 and Iteration 1 OpenAPI contract baseline for identity/invite/auth.
- 2026-03-04: Locked group DM E2EE as required in MVP.
- 2026-03-04: Locked discovery abuse baseline as signed registry + rate limiting + denylist support.
- 2026-03-04: Locked recovery setup as mandatory during onboarding.
- 2026-03-04: Locked MVP UI behavior authority to per-flow state tables in sprint boards.
- 2026-03-04: Locked global server and contact hub pages as first-class navigation surfaces for MVP.
- 2026-03-04: Locked dual server-navigation mode (sidebar + topbar tabs) and burger collapse behavior in server workspace.
- 2026-03-04: Execution hardening aligned E2EE scope across plan/risk/iteration tasks and introduced contract/dependency gates.
- 2026-03-04: Locked server invite policy allowing non-expiring multi-use invite links as an open-access pattern.
- 2026-03-04: Locked privacy-first social policy: mediated friend requests, no default key/profile scraping, and opt-in DM permissions.
- 2026-03-04: Locked DM transport to direct user-to-user channels; guild servers do not relay/store DM payloads.
- 2026-03-04: Locked MVP DM offline policy to best-effort online delivery with encrypted local outbox retries.

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
- Direct user-to-user add flow via expiring contact invite link or QR.
- Friend requests via servers are intent-based and mediated; raw key/profile-identifying data is not exposed by default.
- Dedicated global hubs for browsing servers and contacts with searchable card views.
- Server workspace supports sidebar navigation and topbar tab navigation, including saved tabs and folders.
- Server navigation UI can be shrunk/hidden via burger toggle during focused interaction.
- DM and group DM messaging (edits, mentions, deletes, replies).
- Servers (guilds), text channels, role/permission v1.
- Voice channels, 1:1 calls, and screen share.
- Attachments upload/download (operator-configurable quotas, no global product cap).
- Node-owner controls: local kick/ban/logging tools (no global moderation authority).
- E2EE DMs (1:1 and group DM required in MVP).
- Data ownership v1: full export and import.

### Out of Scope (Post-MVP)

- End-to-end encrypted rooms by default.
- Full decentralized DHT discovery.
- Cross-server DMs.
- Multi-region active-active global architecture.
- Video conferencing parity with enterprise platforms.
- Bot/plugin ecosystem.

### Post-MVP Discovery Roadmap

- Phase A (federated discovery hardening):
  - Keep default registry discovery and optional custom registries.
  - Add trusted-registry scopes for friend/community-only visibility.
- Phase B (hybrid discovery):
  - Support simultaneous federation + P2P discovery.
  - Node/user discoverability policies remain explicit (`private`, `trusted`, `public`).
- Phase C (full P2P optional path):
  - Add decentralized discovery (DHT/gossip style) as an opt-in mode.
  - Preserve self-hosting and privacy controls so users/servers can stay non-global if desired.

## 4) Identity, Auth, and Profile Portability (Locked)

### Design Principles

- No centralized identity provider.
- No required OAuth.
- Node-local membership, user-owned global identity.
- Client-side key ownership and signed profile portability.

### Invite and Join Flow

- Server owner chooses invite mode: one-time or multi-use.
- Invite token fields in MVP: mode, optional expiration, optional max_uses, issuer, and opaque token id.
- Invite scope in MVP is limited to join eligibility only (no role/channel/grant scopes).
- Invite link contains node endpoint, node fingerprint, and token.
- Non-expiring multi-use invite links are allowed and represent an intentionally open join policy.
- Client verifies node fingerprint before redeeming token.
- On first join, client binds its public key to the server membership record.

### User Contact Invite Flow (MVP)

- Users can generate direct contact invites for adding friends without relying on global discovery.
- Contact invite formats:
  - Shareable link (`hexrelay://contact-invite/<token>`)
  - QR payload encoding the same token.
- Contact invite token requirements:
  - Expiration required.
  - One-time by default; optional bounded max-uses for trusted sharing.
  - Bound to inviter identity id and signature metadata.
- Redeem behavior:
  - Recipient redeems token and sees inviter identity preview.
  - Accepting invite creates a friend request or direct friend edge per user settings.
  - Expired/invalid/exhausted tokens fail with deterministic error codes.

### Privacy-First Social Graph Policy (MVP)

- Servers do not expose raw key/profile-identifying data to other users by default.
- Friend requests through a server are mediated actions:
  - User A requests contact with User B through server-local reference.
  - Server sends request notification to User B.
  - Only after User B accepts, both sides receive bootstrap material required for direct relationship setup.
- DM policy defaults to opt-in:
  - Default allow-list: friends/accepted requests only.
  - Per-user override options: allow DMs from same-server members or from anyone.

### Encryption Model

- Transport: TLS for all client/server and server/server channels.
- At-rest: database and blob encryption for node-stored data.
- DMs: E2EE with forward secrecy over direct user-to-user channels (no guild server relay/storage for DM payloads).
- Offline delivery policy (MVP): best-effort online delivery only; sender keeps encrypted local outbox and retries when recipient comes online.
- Keys: private keys remain client-controlled and encrypted at rest on device.

### Authentication Flow

- Server sends a nonce challenge.
- Client signs nonce with private key.
- Server verifies signature against bound public key and issues session token.
- Subsequent logins use challenge-response; no centralized credentials.

### MVP Crypto Profile v1 (Execution Baseline)

- Identity signing key: Ed25519.
- Session key exchange baseline: X25519 + HKDF-SHA256.
- Symmetric encryption baseline for E2EE DM payloads: XChaCha20-Poly1305.
- Signature payload canonicalization: UTF-8 JSON canonical form with sorted keys.
- Challenge nonce requirements: at least 96 bits entropy, single-use, 60-second TTL.
- Replay protection: nonce id persisted until TTL expiry; duplicate nonce use is rejected.
- Key rotation baseline: rotate DM session keys every 100 messages or 24 hours, whichever comes first.
- Error contract baseline: cryptographic verification failures return explicit `*_invalid` codes with no secret-bearing detail.

### Iteration 1 OpenAPI Contract Baseline

- Publish these endpoints before parallel API/web implementation starts:
  - `POST /v1/identity/keys/register`
  - `POST /v1/auth/challenge`
  - `POST /v1/auth/verify`
  - `POST /v1/auth/sessions/revoke`
  - `POST /v1/invites`
  - `POST /v1/invites/redeem`
- Required shared error code set for Iteration 1:
  - `invite_invalid`
  - `invite_expired`
  - `invite_exhausted`
  - `fingerprint_mismatch`
  - `nonce_invalid`
  - `signature_invalid`
  - `session_invalid`
- Contract freeze rule: once Week 2 starts, schema changes require explicit changelog entry in `docs/planning/05-iteration-log.md`.
- Canonical artifact path: `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`.
- Artifact gate: `T2.*` tasks are `blocked` until this contract is committed and referenced in the sprint board.

### Discovery Abuse-Control Baseline (MVP)

- Signed registry metadata remains the source for discovery entries.
- Discovery query paths must enforce per-node rate limits.
- Nodes must support a denylist mechanism for discovery abuse response.

### Portable Profile Capsule

- Public identity card (signed): display name, avatar pointer/hash, bio, metadata.
- Private profile capsule (encrypted client-side): preferences and private metadata.
- Servers store signed public profile and encrypted private blob replicas.
- Profile updates are versioned and signed; newest valid version wins.
- Profile authority rule: user-signed profile data is canonical; server copies are cache/replica only and must not override valid signed user profile fields.

### Recovery and Multi-Device

- Recovery phrase setup is mandatory during onboarding before account setup is considered complete.
- Device-based key backup remains supported as an additional recovery method.
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
- Migration conflict precedence: for profile fields, newest valid signed user state wins; server-owned security and membership enforcement fields remain server-authoritative.
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
- S2.2: As a server owner, I can issue secure invite links with TTL and usage mode.
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
- S4.6: As a user, my group DM messages are E2EE by default.

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

## 9) Execution Task Ownership

- Strategy-level sequencing stays in this document through iterations, epics, and stories.
- Canonical task catalog and task-level status live in `docs/planning/iterations/README.md` and the referenced sprint boards.
- When a story changes scope, update this document first, then update the affected sprint board(s) in the same PR.

## 10) Acceptance Criteria and Definition of Done

### Story-Level DoD

- Unit/integration tests for core logic and edge cases.
- API contracts documented in OpenAPI and validated in CI.
- Permission and auth checks covered by tests.
- Observability added for each new critical path.
- Identity and invite token secrets never appear in logs.
- No plaintext DM content is persisted on servers.

### MVP Exit Criteria

- Canonical KPI definitions and thresholds are maintained in `docs/product/02-prd-v1.md`.
- This plan tracks delivery sequencing; do not update KPI thresholds here.

## 11) Risks and Mitigations

- Canonical dependency and risk register: `docs/product/04-dependencies-risks.md`.
- Keep this section as a directional summary; update the risk register for status/severity changes.

## 12) Immediate Next Actions

- Start Iteration 1 with E1 and E2 only.
- Freeze API contracts before Iteration 2 UI expansion.
- Run end-of-iteration demos with pass/fail against story acceptance.
- Execute and track delivery via the iteration index: `docs/planning/iterations/README.md`.

## 13) Related Documents

- `README.md`
- `docs/product/02-prd-v1.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/README.md`
- `docs/reference/glossary.md`
