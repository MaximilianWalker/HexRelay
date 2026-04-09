# HexRelay PRD v1

## Document Metadata

- Doc ID: prd-v1
- Owner: Product maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-06
- Source of truth: `docs/product/02-prd-v1.md`

## Quick Context

- Primary edit location for product requirements and success metrics.
- Keep locked decisions in `docs/product/01-mvp-plan.md` and reference them here.
- Keep dependency and risk status in `docs/product/04-dependencies-risks.md`.
- `Status: ready` marks this PRD as canonical requirements authority; operational release readiness still depends on unresolved `watch` items in `docs/operations/readiness-corrections-log.md`.
- Latest meaningful change: 2026-04-06 clarified MVP DM reliability requirements around durable acceptance, bounded eventual catch-up, and reachability degradation.

## Product Summary

HexRelay is an open-source, self-hostable communication platform with a modern Discord-like experience and strong user ownership guarantees. The MVP focuses on portable identity, encrypted direct messaging, high-quality voice/screen share, and reliable migration across devices and servers without centralized account infrastructure.

## Runtime and Deployment Model

- Primary product target is a downloadable desktop app that can run off-grid.
- Desktop distribution bundles UI with local API and realtime runtime components.
- Local runtime allows UI launch either inside desktop shell or in a local browser session on localhost.
- Dedicated server deployments are supported as an optional operator mode.
- Browser-only hosted usage is a compatibility mode, not the core product assumption.
- Runtime term mapping is canonical in `docs/reference/glossary.md`.

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

## Locked Product Decisions

- Canonical source: `docs/product/01-mvp-plan.md` section "1.1) Locked Product Decisions (Founder Input)".
- This PRD references those decisions and does not duplicate them to avoid drift.
- Any change to a locked decision must be updated in `docs/product/01-mvp-plan.md` first, then reflected here only where requirement wording depends on it.

## In Scope (MVP)

- Portable key-based identity and node-local membership binding.
- Secure invite links with expiration, max-uses, and one-time/multi-use modes.
- Challenge-signature authentication and session management.
- Friends graph, presence, block/mute, user discovery.
- Global server and contact hub pages with searchable card views.
- DMs/group DMs, servers/channels/roles, edits, mentions, replies.
- E2EE 1:1 and group DMs with rotation and metadata-safe storage behavior.
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

## Post-MVP Discovery Direction

- Keep federation discovery as a supported path (default registry + optional custom registries).
- Add trusted-registry scopes for friend/community-only discoverability.
- Add optional full P2P discovery mode later, while preserving private/invite-only operation.

## Key User Flows

### 1) Create Identity and Join a Server

1. User creates/imports key-based identity.
2. User opens invite link containing node endpoint, fingerprint, and invite token.
3. Client verifies node fingerprint.
4. Invite is redeemed (mode checks, and expiration/max-use checks when configured).
5. Public key is bound to node membership.
6. User enters server with profile hydrated from portable profile data.

### 2) Authenticate on Existing Node

1. Server sends nonce challenge.
2. Client signs challenge with private key.
3. Server verifies signature and issues session token.
4. Session can be listed/revoked from account settings.

### 3) E2EE Direct Messaging

1. Users establish contact bootstrap material through signed out-of-band pairing (invite link, QR, or short code).
2. Client validates pairing signature, nonce/replay, and expiry, then exchanges DM encryption material.
3. Client encrypts outbound DM payloads.
4. Payload is transported over direct user-to-user channel (not via guild server relay and without infra-assisted NAT traversal dependency).
5. Recipient client decrypts locally.
6. If recipient is offline or currently unreachable, sender still keeps the accepted message in canonical DM history and delivery metadata drives bounded eventual catch-up on later reconnect.
7. If any recipient device receives first, profile-linked sibling devices converge via active fanout or deferred catch-up when they later become active.

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

### 6) Navigate Servers and Contacts at Scale

1. User opens global `Servers` hub to browse all joined servers in a searchable card grid.
2. User can pin/favorite servers and group them into folders for sidebar ordering.
3. User can switch server navigation mode between sidebar list/folders and topbar tabs (browser-like).
4. User can save/pin topbar tabs and organize frequent destinations in folders.
5. User can toggle a burger control to collapse/hide server navigation while focused in a server.
6. User opens global `Contacts` hub to browse DMs/friends in a searchable card/list view.
7. User jumps from hub cards directly into the selected server or DM context.

### 7) Add a User Directly by Invite

1. User generates a contact invite link or QR from contacts UI.
2. Invite token includes inviter identity binding, expiration, and usage policy.
3. Recipient redeems token and sees inviter preview.
4. Recipient accepts and a friend request/edge is created.
5. Invalid/expired/exhausted tokens fail with explicit error feedback.

### 8) Send Friend Request Through a Server (Mediated)

1. User A requests contact with User B using server-local reference.
2. Server sends request notification to User B without exposing raw key/profile-identifying data to User A.
3. User B accepts or declines.
4. On accept, both users receive only the bootstrap data required for direct relationship setup.

## Functional Requirements

- Identity
  - Multi-persona support per device.
  - Key generation/import/export with secure local storage.
- Invites
  - One-time or multi-use; expiration and max-uses are optional policy controls.
  - Non-expiring multi-use links are allowed for intentionally open-access servers.
  - MVP invite scope is join eligibility only (no role/channel scoped grants).
- User contact invites
  - Users can generate expiring contact invite links or QR payloads.
  - One-time by default, optional bounded max-uses.
  - Invite payloads are signed and include nonce/expiry for replay-safe pairing bootstrap.
  - Redeem flow must return deterministic error states (`invite_invalid`, `invite_expired`, `invite_exhausted`).
- Friend requests and identity exposure
  - Server-mediated friend request flow is required for in-server user discovery paths.
  - Raw key/profile-identifying data is not exposed to other users by default.
  - Bootstrap identity material is shared only after request acceptance.
- Messaging
  - DMs/group DMs and server channels with edits, mentions, and replies.
  - Moderation-visible edit history applies to server channels, not direct DMs.
  - Guild/community servers do not relay or store DM payloads.
  - DM connectivity must not depend on STUN, TURN, relay, or other always-on third-party/project-operated connectivity services.
  - DM runtime must expose deterministic connection failure reason codes with guided remediation actions.
  - DM send success must mean durable sender-side acceptance into canonical DM history, not merely an attempted live fanout.
  - Default DM policy allows incoming DMs only from friends/accepted requests.
  - Per-user override options: allow same-server members or anyone.
  - Incoming DM payloads must converge across all devices linked to a profile, including devices activated after first delivery.
- DM connectivity execution model
  - Direct-only transport enforcement is required.
  - Out-of-band signed pairing (QR/link/short code) is required.
  - Connectivity preflight and troubleshooter states are required for failed direct attempts.
  - LAN fast path (mDNS/multicast), WAN direct wizard (UPnP/NAT-PMP/manual), and multi-endpoint parallel dial are in-scope reliability enhancers.
  - Multi-device DM convergence requires active-device fanout plus per-device cursor replay/catch-up and idempotent dedupe.
  - Delivery failures must downgrade current reachability assumptions without discarding accepted messages.
  - If direct connectivity cannot be established, product must fail explicitly with user guidance; infra fallback is out of scope.
- Server communication multi-device convergence
  - Channel and presence events must fan out to all active devices linked to the authenticated profile.
  - Devices activated later must hydrate missed channel/presence state by per-device cursor.
  - Reconnect and late-activation hydration must preserve deterministic ordering and dedupe semantics.
- Navigation and Information Architecture
  - Discord-like overall layout and interaction model are baseline.
  - Server navigation must not rely on small circular icon rails as the primary pattern.
  - Global `Servers` and `Contacts` hubs are required and act as first-class navigation surfaces.
  - Server navigation must support both sidebar and topbar tab modes.
  - Topbar navigation must support saved tabs and folder organization.
  - A burger control must allow collapsing/hiding server navigation in server workspace.
- Voice
  - Competitive baseline quality and screen share support.
- Discovery
  - Global and shared-server user discovery.
  - Server discovery supports public listings; private hidden by default.
  - Discovery endpoints enforce rate limiting and node-level denylist controls.
- Data Portability
  - Export/import and full migration paths with integrity verification.
  - Canonical profile data comes from valid user-signed profile state; server replicas are non-authoritative for profile-field precedence.

## Non-Functional Requirements

- Reliability
  - Stable reconnect and event ordering behavior.
- Security
  - TLS everywhere, encrypted at rest for node data, E2EE for 1:1 and group DMs.
  - No key/invite secret leakage to logs.
  - Nonce challenge is single-use with strict TTL and replay rejection.
- Onboarding
  - Recovery phrase setup is mandatory before onboarding completion.
- Performance
  - Realtime and voice SLOs tracked in dashboard.
- Operability
  - Docker Compose-first deployment; simple dashboard first.

## Architecture Summary

- Frontend: Next.js + TypeScript.
- Backend: Rust services (`axum`, `tokio`, `sqlx`, `serde`, `tracing`).
- Infra: PostgreSQL, Redis, S3-compatible storage, WebRTC + coturn.
- Hosting/runtime: local desktop-bundled services by default, with optional dedicated node deployments on local hosts or VPS.

## Success Metrics (MVP)

- Message delivery p95 < 300ms (same region).
- Voice join success > 98%.
- Screen share success > 95% in beta.
- Challenge-signature auth success > 99.5%.
- E2EE DM decrypt success > 99.5% for active sessions.
- Full migration success in at least 3 cross-device test scenarios.

## Risks and Mitigations

- Canonical risk register: `docs/product/04-dependencies-risks.md`.
- This section stays intentionally short to avoid duplicate risk tracking.

## Delivery References

- Master plan: `docs/product/01-mvp-plan.md`
- Iteration boards index: `docs/planning/iterations/README.md`
- DM connectivity proposals: `docs/product/10-infra-free-dm-connectivity-proposals.md`
- DM connectivity execution plan: `docs/planning/infra-free-dm-connectivity-execution-plan.md`

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/reference/glossary.md`
