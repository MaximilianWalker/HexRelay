# HexRelay MVP Plan

HexRelay is an open-source, Discord-like communication platform built for user control: free core features, self-hostable nodes, and a phased path to decentralized discovery and federation.

## Document Metadata

- Doc ID: mvp-plan
- Owner: Product and architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/product/01-mvp-plan.md`

## Quick Context

- Primary edit location for product intent, constraints, architecture baseline, and epics/stories.
- Iteration task sequencing and task-level status are canonical in `docs/planning/iterations/README.md`.
- Dependency/risk severity updates are canonical in `docs/product/04-dependencies-risks.md`.
- `Status: ready` marks this document as the canonical planning authority; release/go-no-go interpretation must still check open `watch` items in `docs/operations/readiness-corrections-log.md`.
- Latest meaningful change: 2026-05-20 accepted the server-node authority model: one user-facing server maps to one separately runnable node/runtime authority; Servers and Contacts navigation decisions remain in progress.

## 1) Product Intent and Constraints

- Free core forever: friends, DMs, servers/channels, voice, file sharing.
- Open source first: no lock-in to a central hosted platform.
- Hybrid operation: each server node can run locally, inside a LAN, privately online, or publicly discoverable by opt-in policy and participate in the server-node P2P network; federation/discovery evolves in phases.
- Server authority model: one user-facing server maps to one separately runnable server runtime/node with its own node identity and state boundary.
- Fast, modern UX: desktop-first distribution with reusable web UI surfaces.
- Rust-first backend: performance, safety, and long-term maintainability.

## 1.1) Locked Product Decisions (Founder Input)

- Target audience for beta: broad communities (gamers, dev groups, private communities).
- Multiple personas/accounts per device are supported in MVP.
- Profile model: global profile by default, optional per-server overrides.
- User discovery: server/node-mediated user discovery supported globally and from shared servers.
- User identity is portable and is not owned by or permanently assigned to a single primary server.
- DM plaintext and private keys remain client/device-only; server nodes/message nodes in the server-node P2P network may carry and store only E2EE DM envelopes plus minimal delivery metadata.
- Message defaults: edits and mentions are required; retention default is forever and configurable per server.
- Moderation model: no centralized platform moderation; only node-owner controls.
- Privacy baseline: encrypted transport and at-rest encryption everywhere; E2EE DMs in MVP.
- DM delivery baseline: encrypted-envelope store-and-forward through server nodes/message nodes in the server-node P2P network only; DM delivery must not use recipient-device LAN/WAN transport.
- Server-node networking is a dynamic policy graph. Discovery, peering, relay, delivery, and storage permissions are separate.
- The user app may supervise or connect to multiple server nodes, but it is not the authority for many unrelated servers inside one app-owned database.
- Two servers hosted on the same physical machine still behave as distinct server instances/nodes with separate identity, configuration, policy, and state.
- Server discovery is opt-in. Online servers can still be private, non-discoverable, invite-only, or non-relaying.
- Users may introduce servers to other servers only when the introduced server descriptor allows user-consented introduction and the user explicitly consents.
- Servers may refuse discovery, peering, relay, or DM forwarding independently.
- Dedicated server administration uses the normal HexRelay app connected to a local or remote node. Dedicated server runtime remains headless and does not get a separate server-specific UI by default.
- UX approval gate: no UX flow, copy, control, or behavior change may be implemented until the user explicitly consents to it.
- Voice target: competitive quality; screen share included in MVP.
- UI direction: heavily Discord-inspired interaction model, except server navigation uses scalable list/card paradigms (no small circular server rail).
- Server navigation supports dual mode: sidebar list/folders and topbar tabbed navigation (browser-like tabs/saved tabs).
- Server navigation uses explicit sidebar/topbar switching plus collapse controls; no burger control is used for Iteration 2.
- File handling: no product-level hard file size cap in MVP (server operators may set local quotas).
- Migration default keeps old device active; optional explicit device revoke is available.
- Bot ecosystem deferred until post-MVP; core stability first.
- License choice: AGPL-3.0.
- Contribution policy: DCO sign-off required, no CLA for MVP.

## 1.2) Decision Log

- 2026-03-03: Added docs metadata and decision-log section to standardize documentation governance.
- 2026-03-04: Locked MVP invite scope semantics to mode + expiration + max-uses only; role/channel scoped invites deferred post-MVP.
- 2026-03-04: Added MVP Crypto Profile and Iteration 1 OpenAPI contract baseline for identity/invite/auth.
- 2026-03-04: Locked group DM E2EE as required in MVP.
- 2026-03-04: Locked discovery abuse baseline as signed registry + rate limiting + denylist support.
- 2026-03-04: Locked recovery setup as mandatory during onboarding.
- 2026-03-04: Locked MVP UI behavior authority to `docs/product/08-screen-state-spec.md` with sprint-board execution mappings.
- 2026-03-04: Locked global server and contact hub pages as first-class navigation surfaces for MVP.
- 2026-03-04: Locked dual server-navigation mode (sidebar + topbar tabs) and collapse behavior in server workspace; the burger-control detail was superseded on 2026-05-20.
- 2026-03-04: Execution hardening aligned E2EE scope across plan/risk/iteration tasks and introduced contract/dependency gates.
- 2026-03-04: Locked server invite policy allowing non-expiring multi-use invite links as an open-access pattern.
- 2026-03-04: Locked privacy-first social policy: mediated friend requests, no default key/profile scraping, and opt-in DM permissions.
- 2026-03-04: Initially locked an infrastructure-free DM transport idea; superseded by the 2026-05-08 encrypted-envelope delivery decision.
- 2026-03-04: Locked MVP DM offline policy to durable sender-side acceptance with bounded eventual catch-up and explicit delivery-state tracking rather than best-effort-only online delivery.
- 2026-03-04: Locked deployment model to bundled desktop local-first runtime with optional dedicated server mode.
- 2026-03-12: Initially locked DM connectivity to infrastructure-free client paths only; superseded by the 2026-05-08 encrypted-envelope delivery decision.
- 2026-03-12: Locked profile-device convergence requirement: incoming communication must sync to all profile devices, including devices that become active after first delivery.
- 2026-05-07: Locked Windows and Linux as first-class release targets, Tauri as the default desktop shell, and dedicated server delivery as a separate service/package family from the desktop installer.
- 2026-05-08: Locked server-node/message-node E2EE envelope delivery as the MVP DM baseline. Servers may store and forward ciphertext envelopes and minimal delivery metadata only; node-bypassing client DM transport/bootstrap surfaces are out of scope.
- 2026-05-11: Clarified that server runtimes act as peers in the server-node P2P network for DM envelope delivery, and broadened the explicit user-approval gate from DM delivery UX to all UX decisions.
- 2026-05-11: Locked the server-node P2P architecture direction as a dynamic opt-in policy graph with no primary-server assumption, private/LAN/local-only node support, user-consented node introductions, and separate discovery/peering/relay/delivery/storage permissions.
- 2026-05-11: Locked dedicated-server administration to the normal HexRelay app for authorized node owners/admins; dedicated server packages stay headless and no separate server-specific UI ships by default.
- 2026-05-11: Added T4.1.8 backend retention and abuse controls for encrypted-envelope delivery metadata: 30-day fanout metadata retention, 7-day outbound forwarding metadata retention, and per-sender/device/node rate limits without plaintext inspection.
- 2026-05-11: Added T4.1.9 backend realtime dispatch summaries for encrypted DM active-device fanout; summaries classify target-device routing outcomes but final delivery remains recipient-device ack-backed and no UX changes were introduced.
- 2026-05-20: Recorded in-progress Iteration 2 UX decisions that Servers and Contacts hubs share card/list layouts, shared filters, selection, and action-menu behavior, while desktop navigation uses sidebar/topbar switching plus collapse controls without a burger control.
- 2026-05-20: Locked the server-node authority decision: one user-facing server equals one separately runnable server runtime/node authority; current multi-server-in-one-API storage is transitional scaffolding only.

## 1.3) Runtime and Deployment Modes (Locked)

- Primary mode is a downloadable desktop app where each user can run HexRelay off-grid.
- Windows and Linux are both first-class desktop release targets.
- Desktop packaging uses Tauri by default and bundles UI plus local API/realtime runtime components for user-local operation.
- Dedicated server mode is supported for operators who want headless hosting.
- Dedicated server delivery is a separate service/package family from the desktop installer.
- Dedicated-server administration is app-mediated: authorized node owners/admins connect through the normal HexRelay app to local, LAN, private online, or public node endpoints.
- Dedicated server packages may expose authenticated admin/operator APIs, but no separate server-specific UI artifact is assumed for MVP.
- Runtime remains multi-component (`apps/web`, `services/api-rs`, `services/realtime-rs`) even when desktop packaging installs and supervises local runtime components.
- A single API runtime is not the canonical authority for many user-facing servers. The app aggregates joined server nodes; each server/node owns its own authority boundary.
- Browser-only usage is a compatibility path, not the primary runtime target.
- Terminology mapping for runtime words (`node`, `server`, `dedicated server`) is canonical in `docs/reference/glossary.md`; server/node authority is canonical in `docs/architecture/adr-0004-server-node-authority.md`.
- Release artifact details and code signing expectations are canonical in `docs/operations/03-release-packaging.md`.

## 2) Architecture Decision (Locked)

### Frontend

- Web app: Next.js + TypeScript.
- Why: reusable UI layer for desktop shell distribution and optional browser compatibility.

### Backend/Core

- Rust for API/realtime/federation services.
- Why: memory safety + high concurrency + strong protocol ergonomics.

### Data and Infra

- PostgreSQL: source of truth.
- Redis: presence, ephemeral state, fanout cache.
- S3-compatible storage: media blobs.
- WebRTC + coturn: voice/call media path.
- Server/node identity: one server authority per runtime node; transitional `servers` rows represent the connected local node's own server state until schema cleanup converges naming.

## 3) MVP Functional Scope

### In Scope

- Portable identity (public/private key), profile, device sessions.
- Multiple personas/accounts on one device.
- Friends graph: add/remove/block/mute + presence.
- User discovery (global and shared-server contexts) with server-mediated contact requests.
- Contact invite add flow via expiring node-mediated contact invite link.
- Friend requests via servers are intent-based and mediated; raw key/profile-identifying data is not exposed by default.
- Dedicated global hubs for browsing servers and contacts with searchable card and list layouts.
- Server workspace supports sidebar navigation and topbar tab navigation, including saved tabs and folders.
- Server navigation UI can be collapsed and switched between sidebar and topbar modes without a burger control.
- DM and group DM messaging (edits, mentions, deletes, replies).
- E2EE DM delivery stack: ciphertext-envelope node delivery, relationship-scoped contact/encryption bootstrap, delivery metadata minimization, and deterministic delivery-state UI.
- Profile-device eventual-sync stack: active-device fanout plus late-device replay/catch-up by per-device cursor for DM and server communication.
- Servers, text channels, role/permission baseline.
- Voice channels, 1:1 calls, and screen share.
- Attachments upload/download (operator-configurable quotas, no global product cap).
- Node-owner controls: local kick/ban/logging tools (no global moderation authority).
- E2EE DMs (1:1 and group DM required in MVP).
- Data ownership baseline: full export and import.

### Out of Scope (Post-MVP)

- End-to-end encrypted rooms by default.
- Full decentralized DHT discovery.
- Full arbitrary cross-server DMs beyond the policy-controlled encrypted-envelope architecture.
- Multi-region active-active global architecture.
- Video conferencing parity with enterprise platforms.
- Bot/plugin ecosystem.

### Post-MVP Discovery Roadmap

- Phase A (private and trusted node discovery):
  - Keep static peers, signed peer invite tokens, signed node descriptors, default registry discovery, and optional custom registries.
  - Add trusted scopes for friend/community-only visibility.
  - Preserve explicit node modes: local-only, LAN-only, private online, member-visible, trusted registry, and public opt-in.
- Phase B (hybrid server-node discovery):
  - Support simultaneous default registry, custom registry, allowlisted server-to-server discovery, and user-consented node introductions.
  - Node/user discoverability policies remain explicit and independent from peering, relay, delivery, and storage permissions.
  - User introductions create candidate peers only; each server still validates descriptors and may refuse.
- Phase C (decentralized discovery path):
  - Add decentralized server/node discovery as an opt-in mode only after abuse controls, revocation, and privacy boundaries are mature.
  - Evaluate Kademlia-style DHT for signed descriptor lookup only, HyParView-style peer sampling for larger opt-in overlays, and Plumtree-style gossip only for low-sensitivity node metadata.
  - Preserve self-hosting and privacy controls so users/servers can stay local, LAN-only, private, or non-global if desired.

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
- Invite redemption joins the server node identified by that endpoint and fingerprint. It must not create an unrelated server row inside the current app database.
- Non-expiring multi-use invite links are allowed and represent an intentionally open join policy.
- Client verifies node fingerprint before redeeming token.
- On first join, client binds its public key to the server membership record.

### User Contact Invite Flow (MVP)

- Users can generate contact invites for adding friends without relying on global discovery.
- Contact invite formats:
  - Shareable link (`hexrelay://contact-invite/<token>`)
- Contact invite token requirements:
  - Expiration required.
  - One-time by default; optional bounded max-uses for trusted sharing.
  - Bound to inviter identity id and signature metadata.
- Redeem behavior:
  - Recipient redeems token and sees inviter identity preview.
  - Accepting invite creates a server-mediated friend request or accepted friend edge per user settings.
  - Expired/invalid/exhausted tokens fail with deterministic error codes.

### DM Delivery Model (MVP Locked)

- DM relationship and encryption bootstrap uses accepted contact-invite redemption or accepted mediated friend-request bootstrap.
- Bootstrap material includes only the identity key and profile-device data required for client-side E2EE setup; recipient-device network endpoint hints, DM pairing QR payloads, and manual-code bootstrap are out of scope.
- Normal DM send success uses the server-node P2P encrypted-envelope path and must not require recipient-device network reachability.
- Server nodes/message nodes may carry and store only E2EE DM envelopes plus minimal delivery metadata needed for authorization, routing, dedupe, delivery state, retention, and abuse controls.
- Delivery metadata retention is separate from canonical encrypted DM history: expired replay/forwarding metadata can be purged without deleting accepted ciphertext history.
- Abuse controls are metadata-only and policy-only: dispatch is sender scoped, catch-up is identity scoped across profile devices, ack is identity/device scoped, and authenticated node-forward ingress is origin-node scoped.
- Active-device realtime fanout exposes backend-only target summaries for queued-to-verified-websocket, pending/no-connection, unverified device binding, saturated queue, and stale-connection cleanup outcomes.
- Live dispatch summaries are not user-visible read or final-delivery state; final delivery remains `dm.envelope.ack` backed and late-device catch-up remains the deterministic fallback.
- Origin, delivery, relay, and discoverable node roles are selected by current node policy and route availability; no node role implies ownership of a user's identity.
- Relay is optional and policy-controlled. A server may be discoverable or peered while still refusing relay or DM forwarding.
- DM plaintext, decrypted message views, and private keys remain client/device-only and must never be uploaded for server-side processing.
- Delivery flow order:
  - Validate relationship, block state, DM policy, and bootstrap authenticity/replay/expiry.
  - Client encrypts per-recipient/device envelope payloads before handing them to a server node/message node in the server-node P2P network.
  - Message node durably accepts ciphertext envelopes and minimal delivery metadata before sender-visible success.
  - Active recipient devices receive envelopes through node fanout.
  - Later-active devices catch up through per-device cursors and idempotent replay/dedupe over ciphertext envelopes.
- Delivery acknowledgements and read state are separate: `Delivered` is recipient-device envelope receipt, while participant-visible `Read` requires an explicit read receipt permitted by reader privacy settings.
- Recipient-device LAN discovery, WAN setup wizard, endpoint cards, preflight, pairing QR/manual code, and parallel dial are out of scope for DM delivery.
- Non-goals: server-readable DM content, private-key custody/upload, unencrypted DM mailboxing, or plaintext relay behavior.

### Profile-Device Sync (DM and Server Communication)

- One profile may be active on multiple devices simultaneously.
- Incoming communication must fan out to all currently active profile devices.
- Devices that were offline at first receive must catch up on later activation.
- Convergence contract applies to:
  - DM encrypted-envelope delivery (ciphertext envelope replay across profile devices),
  - server-channel and presence communication (node-authoritative event hydration by per-device cursor).
- Reconciliation baseline:
  - per-device cursor checkpoints are tracked and persisted,
  - duplicate replays are idempotent by stable message/event identity,
  - read-state merge rules are deterministic across profile devices.

### Privacy-First Social Graph Policy (MVP)

- Servers do not expose raw key/profile-identifying data to other users by default.
- Any bidirectional block relationship suppresses peer-facing invite redemption, bootstrap material, and discovery visibility until the block is removed.
- Friend requests through a server are mediated actions:
  - User A requests contact with User B through server-local reference.
  - Server sends request notification to User B.
  - Only after User B accepts, both sides receive bootstrap material required for DM relationship and encryption setup.
- Server-channel message authorship is valid only while the author is a current member of that server; write boundaries must reject non-member authors even for internal callers.
- Server-channel live websocket fanout is currently best-effort after persistence; message history durability is stronger than live delivery guarantees until an outbox/retry design exists.
- Realtime outbound backpressure must degrade delivery without silently unregistering still-open websocket sessions; queue saturation is not equivalent to disconnect.
- Presence and channel profile-device replay should share one private replay-store/cursor implementation where semantics are identical, while keeping domain-specific publish logic separate.
- DM policy defaults to opt-in:
  - Default allow-list: friends/accepted requests only.
  - Per-user override options: allow DMs from same-server members or from anyone.

### Encryption Model

- Transport: TLS for all client/server and server/server channels.
- At-rest: database and blob encryption for node-stored data.
- DMs: E2EE with forward secrecy; server nodes/message nodes handle ciphertext envelopes only, while plaintext and private keys remain client/device-only.
- Offline delivery policy (MVP): sender-side success means durable acceptance of encrypted DM envelopes into canonical DM history plus minimal delivery metadata; live delivery uses node fanout, while later reconnects use bounded eventual catch-up rather than fire-and-forget retry semantics.
- Keys: private keys remain client-controlled and encrypted at rest on device.

### Authentication Flow

- Public identity-key registration must fail closed unless a trusted claim flow exists.
- Server sends a nonce challenge.
- Client signs nonce with private key.
- Server verifies signature against bound public key and issues session token.
- Subsequent logins use challenge-response; no centralized credentials.

### MVP Crypto Profile (Execution Baseline)

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
  - `POST /identity/keys/register`
  - `POST /auth/challenge`
  - `POST /auth/verify`
  - `POST /auth/sessions/revoke`
  - `POST /invites`
  - `POST /invites/redeem`
- Required shared error code set for Iteration 1:
  - `invite_invalid`
  - `invite_expired`
  - `invite_exhausted`
  - `fingerprint_mismatch`
  - `nonce_invalid`
  - `signature_invalid`
  - `session_invalid`
- Contract freeze rule: once Week 2 starts, schema changes require explicit changelog entry in `docs/planning/05-iteration-log.md`.
- Canonical artifact path: `docs/contracts/runtime-rest.openapi.yaml`.
- Artifact gate: `T2.*` tasks are `blocked` until this contract is committed and referenced in the sprint board.

### Discovery Abuse-Control Baseline (MVP)

- Signed registry metadata remains the source for discovery entries.
- Signed node descriptors are the source for server-node discovery claims.
- Discovery query paths must enforce per-node rate limits.
- Nodes must support a denylist mechanism for discovery abuse response.
- Discovery publication is opt-in and policy-scoped; discovery does not imply peering, relay, delivery, storage, membership, or trust.

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
- Auth/session transport: challenge-response with signed nonce, HttpOnly session cookie auth, and double-submit CSRF header for authenticated mutation routes.
- Internal service auth: capability-scoped bearer credentials; presence watcher resolution and channel fanout ingress must not share one broad token.
- Realtime ingress abuse controls: websocket connect rate limiting, per-identity connection cap, inbound message-size cap, and message-rate cap.
- Background jobs/events: `tokio` tasks + Redis streams/pubsub.
- Observability: `tracing`, OpenTelemetry exporter.

### Frontend and Tooling

- Next.js + TypeScript + Tailwind.
- API schema contracts: OpenAPI and generated TS client.
- Monorepo: npm workspace scripts + Rust workspace.
- Key management: WebCrypto for keypair operations and local encrypted key storage.
- Passphrase-gated local key unlock: optional future hardening, not required for MVP baseline.

### Infrastructure

- Docker Compose for local single-node deployment.
- Kubernetes manifests later (post-MVP hardening).
- CI: lint/test/build for Rust + web, plus dependency/SAST security gates and artifacted integration evidence.

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
- E4: DMs/group DMs, server channels, permissions baseline, and E2EE DM baseline.

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

- Canonical KPI definitions and thresholds are maintained in `docs/product/02-prd.md`.
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
- `docs/product/02-prd.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/README.md`
- `docs/reference/glossary.md`
