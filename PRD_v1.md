# HexRelay PRD v1

## Product Summary

HexRelay is an open-source, self-hostable communication platform with a modern Discord-like experience and strong user ownership guarantees. The MVP focuses on portable identity, encrypted direct messaging, high-quality voice/screen share, and reliable migration across devices and servers without centralized account infrastructure.

## Vision

Build a communication stack where users and communities control identity, data location, and hosting model while keeping UX quality competitive with mainstream platforms.

## Product Principles

- User-owned identity, not provider-owned identity.
- No central lock-in for accounts, hosting, or data portability.
- Secure-by-default direct communication (E2EE DMs).
- Practical decentralization: hybrid architecture first, deeper federation later.
- Core stability before ecosystem expansion.

## Target Users (MVP)

- Gaming communities.
- Developer and open-source communities.
- Private friend groups and self-hosters.

## Non-Negotiable MVP Decisions

- Multiple personas/accounts per device.
- Global profile by default with optional per-server overrides.
- Direct user discovery globally and from shared servers.
- No cross-server DMs.
- Message baseline: edits + mentions + core chat primitives.
- Default message retention: forever; configurable by server owner.
- Message edit history visible to node moderators/owners.
- E2EE for 1:1 DMs in MVP.
- Competitive voice quality plus screen share in MVP.
- No global file size cap (node operators can set quotas).
- Migration keeps old device active by default; optional cutover revoke.
- Bot/plugin ecosystem deferred post-MVP.
- License: AGPL-3.0.
- Contributions: DCO sign-off required, no CLA in MVP.

## In Scope (MVP)

- Portable key-based identity and node-local membership binding.
- Secure invite links with expiration and one-time/multi-use modes.
- Challenge-signature authentication and session management.
- Friends graph, presence, block/mute, user discovery.
- DMs/group DMs, servers/channels/roles, edits, mentions, replies.
- E2EE 1:1 DMs with rotation and metadata-safe storage behavior.
- Voice channels, 1:1 calls, and screen share.
- Media upload/download with node-configurable quotas/policies.
- Local node-owner controls (kick/ban/logs).
- Profile capsule replication (signed public + encrypted private replicas).
- Quick restore and full migration flows (LAN and encrypted bundle).
- Basic operational dashboard + SLO alerting for beta.

## Out of Scope (MVP)

- Full DHT-based decentralized discovery.
- Cross-server DMs.
- Full bot/plugin platform.
- Enterprise-grade global active-active architecture.

## Key User Flows

### 1) Create Identity and Join a Server

1. User creates/imports key-based identity.
2. User opens invite link containing node endpoint, fingerprint, and invite token.
3. Client verifies node fingerprint.
4. Invite is redeemed (mode + expiration checks).
5. Public key is bound to node membership.
6. User enters server with profile hydrated from portable profile data.

### 2) Authenticate on Existing Node

1. Server sends nonce challenge.
2. Client signs challenge with private key.
3. Server verifies signature and issues session token.
4. Session can be listed/revoked from account settings.

### 3) E2EE Direct Messaging

1. Users establish DM session and exchange encryption material.
2. Client encrypts outbound DM payloads.
3. Server stores ciphertext and required delivery metadata.
4. Recipient client decrypts locally.

### 4) New Device Restore

1. User scans QR from existing device (quick restore) or uses recovery path.
2. Identity and server list are transferred securely.
3. New device re-auths and syncs server state.
4. Profile capsules restore automatically.

### 5) Full Migration

1. Old device creates encrypted signed migration bundle (`.hxb`).
2. Transfer occurs via LAN direct mode or encrypted file import.
3. New device verifies signature, decrypts, restores state, and reconciles with servers.
4. Optional cutover revokes old-device sessions.

## Functional Requirements

- Identity
  - Multi-persona support per device.
  - Key generation/import/export with secure local storage.
- Invites
  - One-time or multi-use; expiration required.
- Messaging
  - DMs/group DMs/server channels with edits, mentions, and moderation-visible edit history.
- Voice
  - Competitive baseline quality and screen share support.
- Discovery
  - Global and shared-server user discovery.
  - Server discovery supports public listings; private hidden by default.
- Data Portability
  - Export/import and full migration paths with integrity verification.

## Non-Functional Requirements

- Reliability
  - Stable reconnect and event ordering behavior.
- Security
  - TLS everywhere, encrypted at rest for node data, E2EE for 1:1 DMs.
  - No key/invite secret leakage to logs.
- Performance
  - Realtime and voice SLOs tracked in dashboard.
- Operability
  - Docker Compose-first deployment; simple dashboard first.

## Architecture Summary

- Frontend: Next.js + TypeScript.
- Backend: Rust services (`axum`, `tokio`, `sqlx`, `serde`, `tracing`).
- Infra: PostgreSQL, Redis, S3-compatible storage, WebRTC + coturn.
- Hosting: self-hosted local or VPS nodes; public/private listing choice per node.

## Success Metrics (MVP)

- Message delivery p95 < 300ms (same region).
- Voice join success > 98%.
- Screen share success > 95% in beta.
- Challenge-signature auth success > 99.5%.
- E2EE DM decrypt success > 99.5% for active sessions.
- Full migration success in at least 3 cross-device test scenarios.

## Risks and Mitigations

- E2EE complexity risk
  - Limit MVP to 1:1 DM E2EE; defer group E2EE if needed.
- Voice/screen share network variability
  - TURN fallback, diagnostics, and soak tests.
- Migration data integrity risk
  - Signed/encrypted bundles with schema versioning and reconcile checks.
- Scope creep
  - Defer bot ecosystem and advanced federation until after MVP stability.

## Delivery References

- Master plan: `MVP_PLAN.md`
- Iteration boards:
  - `ITERATION_1_SPRINT_BOARD.md`
  - `ITERATION_2_SPRINT_BOARD.md`
  - `ITERATION_3_SPRINT_BOARD.md`
  - `ITERATION_4_SPRINT_BOARD.md`
