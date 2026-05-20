# HexRelay PRD

## Document Metadata

- Doc ID: prd
- Owner: Product maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/product/02-prd.md`

## Quick Context

- Primary edit location for product requirements and success metrics.
- Keep locked decisions in `docs/product/01-mvp-plan.md` and reference them here.
- Keep dependency and risk status in `docs/product/04-dependencies-risks.md`.
- `Status: ready` marks this PRD as canonical requirements authority; operational release readiness still depends on unresolved `watch` items in `docs/operations/readiness-corrections-log.md`.
- Latest meaningful change: 2026-05-20 accepted the server authority model and locked Iteration 2 navigation decisions for shared Servers/Contacts hubs, `Pinned` terminology, explicit desktop navigation controls, and no MVP navigation collections model.

## Product Summary

HexRelay is an open-source, self-hostable communication platform with a modern Discord-like experience and strong user ownership guarantees. The MVP focuses on portable identity, encrypted direct messaging, high-quality voice/screen share, and reliable migration across devices and servers without centralized account infrastructure.

## Runtime and Deployment Model

- Primary product target is a downloadable desktop app that can run off-grid.
- Windows and Linux are both first-class desktop release targets.
- Desktop distribution uses Tauri by default and bundles UI with local API and realtime runtime components.
- Local runtime allows UI launch either inside desktop shell or in a local browser session on localhost.
- Each user-facing server is backed by one server runtime authority with its own server identity and state boundary.
- The desktop app may supervise multiple local server runtimes or connect to multiple remote servers, but the app is not the authority for many unrelated servers inside one shared API database.
- Dedicated server deployments are supported as a separate optional operator service/package mode.
- Dedicated server administration is performed through the normal HexRelay app connected to the server endpoint for identities with server-owner/admin permissions; dedicated server packages remain headless and do not ship a separate server-specific UI by default.
- Browser-only hosted usage is a compatibility mode, not the core product assumption.
- Runtime term mapping is canonical in `docs/reference/glossary.md`; server authority is canonical in `docs/architecture/adr-0004-server-authority.md`.
- Release artifact details and code signing expectations are canonical in `docs/operations/03-release-packaging.md`.

## Vision

Build a communication stack where users and communities control identity, data location, and hosting model while keeping UX quality competitive with mainstream platforms.

## Product Principles

- User-owned identity, not provider-owned identity.
- No central lock-in for accounts, hosting, or data portability.
- Secure-by-default private messaging (E2EE DMs).
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

- Portable key-based identity and server-local membership binding.
- Secure invite links with expiration, max-uses, and one-time/multi-use modes.
- Challenge-signature authentication and session management.
- Friends graph, presence, block/mute, user discovery.
- Global server and contact hub pages with searchable card and list layouts.
- DMs/group DMs, servers/channels/roles, edits, mentions, replies.
- E2EE 1:1 and group DMs with rotation and metadata-safe storage behavior.
- Voice channels, 1:1 calls, and screen share.
- Media upload/download with server-configurable quotas/policies.
- Local server-owner controls (kick/ban/logs).
- Profile capsule replication (signed public + encrypted private replicas).
- Quick restore and full migration flows (LAN and encrypted bundle).
- Basic operational dashboard + SLO alerting for beta.

## Out of Scope (MVP)

- Full DHT-based decentralized discovery.
- Full arbitrary cross-server DMs beyond the policy-controlled encrypted-envelope architecture.
- Full bot/plugin platform.
- Enterprise-grade global active-active architecture.

## Post-MVP Discovery Direction

- Keep federation discovery as a supported path (default registry + optional custom registries).
- Add trusted-registry scopes for friend/community-only discoverability.
- Preserve local-only, LAN-only, private online, invite-only, member-visible, trusted registry, and public opt-in operation.
- Add user-consented server introductions where the introduced server descriptor explicitly permits that sharing.
- Add optional decentralized server discovery later for signed descriptor lookup only, while preserving private/invite-only operation.

## Key User Flows

### 1) Create Identity and Join a Server

1. User creates/imports key-based identity.
2. User opens invite link containing server endpoint, server id, and invite token.
3. Client verifies server id.
4. Invite is redeemed (mode checks, and expiration/max-use checks when configured).
5. Public key is bound to server membership.
6. User enters server with profile hydrated from portable profile data.

### 2) Authenticate on Existing Server

1. Server sends nonce challenge.
2. Client signs challenge with private key.
3. Server verifies signature and issues session token.
4. Session can be listed/revoked from account settings.

### 3) E2EE Private Messaging

1. Users establish contact eligibility through an accepted friend request.
2. API releases only the public identity and profile-device bootstrap material required for client-side E2EE setup after relationship acceptance.
3. Client encrypts outbound DM payloads into per-recipient/device E2EE envelopes.
4. Servers/message servers in the server-to-server network carry and store only ciphertext envelopes plus minimal delivery metadata.
5. Origin, delivery, and optional relay servers are selected by current server policy and route availability; no server role owns the user's identity.
6. Recipient client decrypts locally; server-side plaintext access and private-key custody are forbidden.
7. If recipient is offline or currently unreachable, accepted encrypted envelopes remain in canonical DM history and delivery metadata drives bounded eventual catch-up on later reconnect.
8. If any recipient device receives first, profile-linked sibling devices converge via active fanout or deferred catch-up when they later become active.

### 4) New Device Restore

1. User scans QR from existing device (quick restore) or uses recovery path.
2. Identity and server list are transferred securely.
3. New device re-auths and syncs server state.
4. Profile capsules restore automatically.

### 5) Full Migration

1. Old device creates encrypted signed migration bundle (`.hxb`).
2. Transfer occurs via trusted device-local transfer or encrypted file import.
3. New device verifies signature, decrypts, restores state, and reconciles with servers.
4. Optional cutover revokes old-device sessions.

### 6) Navigate Servers and Contacts at Scale

1. User opens global `Servers` hub to browse all joined servers in searchable card or list layout.
2. User can pin servers and contacts from shared hub actions.
3. User can switch desktop server navigation mode between sidebar list and topbar tabs.
4. User can save/pin topbar tabs and reorder them manually.
5. User can collapse sidebar navigation while focused in a server through an explicit control.
6. User opens global `Contacts` hub to browse DMs/friends in a searchable card/list view.
7. User opens the selected server or DM context from hub cards.

### 7) Add a Contact by Friend Request

1. User searches for another user through approved discovery scopes or enters a known identity id.
2. User sends a friend request.
3. Target user sees the inbound request and accepts or declines.
4. Requester may cancel while the request is pending.
5. On acceptance, only the bootstrap material required for DM relationship and encryption setup is released.

### 8) Send Friend Request Through a Server (Mediated)

1. User A requests contact with User B using server-local reference.
2. Server sends request notification to User B without exposing raw key/profile-identifying data to User A.
3. User B accepts or declines.
4. On accept, both users receive only the bootstrap data required for DM relationship and encryption setup.

## Functional Requirements

- Identity
  - Multi-persona support per device.
  - Key generation/import/export with secure local storage.
- Invites
  - One-time or multi-use; expiration and max-uses are optional policy controls.
  - Non-expiring multi-use links are allowed for intentionally open-access servers.
  - MVP invite scope is join eligibility only (no role/channel scoped grants).
  - Invite redemption binds membership to the endpoint and server id in the invite; it does not create a second independent server inside the current API runtime.
- Friend requests and identity exposure
  - Friend requests are required for user contact add flows.
  - User search and shared-server discovery may initiate requests, but identity/bootstrap material is not trusted until acceptance.
  - Raw key/profile-identifying data is not exposed to other users by default.
  - Bootstrap identity material is shared only after request acceptance.
- Messaging
  - DMs/group DMs and server channels with edits, mentions, and replies.
  - Moderation-visible edit history applies to server channels, not private DMs.
  - Servers/message servers in the server-to-server network may carry and store E2EE DM envelopes plus minimal delivery metadata only.
  - DM plaintext, decrypted views, and private keys must remain client/device-only.
  - DM send success must mean durable sender-side acceptance of encrypted envelopes into canonical DM history plus delivery metadata, not merely an attempted live fanout.
  - Default DM policy allows incoming DMs only from friends/accepted requests.
  - Per-user override options: allow same-server members or anyone.
  - Incoming DM envelopes must converge across all devices linked to a profile, including devices activated after first delivery.
- UX approval
  - No UX flow, copy, control, or behavior change may be implemented until the user explicitly consents to it.
- DM delivery execution model
  - E2EE envelope delivery through servers/message servers in the server-to-server network is the only MVP DM transport path.
  - Server-to-server discovery, peering, relay, delivery, and encrypted storage permissions are separate.
  - Relay is optional and policy-controlled; a server may be discoverable or peered while refusing relay or DM forwarding.
  - Accepted friend-request state is required before encryption material is trusted.
  - Delivery-state diagnostics are required for blocked policy, missing bootstrap, offline recipient, message-server unavailable, and catch-up/replay failures; these diagnostics must not become a DM preflight/troubleshooter UX.
  - Recipient-device pairing QR/manual code, LAN discovery, WAN wizard, endpoint cards, connectivity preflight, and parallel dial are out of scope for DM delivery.
  - Multi-device DM convergence requires active-device fanout plus per-device cursor replay/catch-up and idempotent dedupe over ciphertext envelopes.
  - Unencrypted DM mailboxing, server-side decryption, server-readable DM content, and private-key upload/custody are out of scope.
- Server communication multi-device convergence
  - Channel and presence events must fan out to all active devices linked to the authenticated profile.
  - Devices activated later must hydrate missed channel/presence state by per-device cursor.
  - Reconnect and late-activation hydration must preserve deterministic ordering and dedupe semantics.
- Navigation and Information Architecture
  - Discord-like overall layout and interaction model are baseline.
  - Server navigation must not rely on small circular icon rails as the primary pattern.
  - Global `Servers` and `Contacts` hubs are required and act as first-class navigation surfaces.
  - Server navigation must support both sidebar and topbar tab modes.
  - Topbar navigation must support saved tabs, pinned tabs, and manual reorder.
  - Server workspace navigation must support explicit sidebar/topbar switching plus sidebar collapse.
  - Navigation collection management is out of MVP.
- Server authority
  - One user-facing server maps to one separately runnable server runtime.
  - App-level multi-server views aggregate joined servers.
  - Two servers on the same physical host must still have distinct server identities, configuration, policy, and state boundaries.
  - API paths using `server_id` are scoped to the connected server identity; another server id belongs behind another server endpoint.
- Voice
  - Competitive baseline quality and screen share support.
- Discovery
  - Global and shared-server user discovery.
  - Server discovery is opt-in and descriptor-scoped.
  - Server discovery supports public listings; private, invite-only, LAN-only, and local-only servers remain hidden by default.
  - Online servers may remain private and non-discoverable.
  - User-consented server introductions are allowed only when the introduced server descriptor permits that discovery path and the user explicitly consents.
  - Discovery never implies peering, relay, delivery, storage, membership, or trust.
  - Discovery endpoints enforce rate limiting and server-level denylist controls.
- Data Portability
  - Export/import and full migration paths with integrity verification.
  - Canonical profile data comes from valid user-signed profile state; server replicas are non-authoritative for profile-field precedence.

## Non-Functional Requirements

- Reliability
  - Stable reconnect and event ordering behavior.
- Security
  - TLS everywhere, encrypted at rest for server data, E2EE for 1:1 and group DMs.
  - Servers/message servers store and forward DM ciphertext envelopes only; plaintext and private keys stay client/device-only.
  - No key/invite secret leakage to logs.
  - Nonce challenge is single-use with strict TTL and replay rejection.
- Onboarding
  - Recovery phrase setup is mandatory before onboarding completion.
- Performance
  - Realtime and voice SLOs tracked in dashboard.
- Operability
  - Docker Compose-first deployment; simple dashboard first.
  - Dedicated server administration uses authenticated app-to-server management surfaces rather than a separate dedicated-server UI artifact by default.

## Architecture Summary

- Frontend: Next.js + TypeScript.
- Backend: Rust services (`axum`, `tokio`, `sqlx`, `serde`, `tracing`).
- Infra: PostgreSQL, Redis, S3-compatible storage, and WebRTC + coturn for voice/call media only.
- Hosting/runtime: local desktop-bundled services by default, with optional dedicated server deployments on local hosts, LANs, or VPS; one user-facing server maps to one server runtime, server runtimes are the peers in the server-to-server network, and clients attach to servers.
- Dedicated-server administration: app-mediated for authorized server owners/admins, with authenticated operator/admin APIs and no standalone server-specific UI artifact by default.
- Server-to-server topology: dynamic opt-in policy graph with portable user identity, no primary-server assumption, and separate discovery/peering/relay/delivery/storage permissions.

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
- Release packaging: `docs/operations/03-release-packaging.md`
- DM envelope delivery proposals: `docs/product/10-infra-free-dm-connectivity-proposals.md`
- DM envelope delivery execution plan: `docs/planning/infra-free-dm-connectivity-execution-plan.md`

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/operations/03-release-packaging.md`
- `docs/reference/glossary.md`
