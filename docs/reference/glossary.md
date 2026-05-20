# HexRelay Glossary

## Document Metadata

- Doc ID: glossary
- Owner: Product and engineering maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/reference/glossary.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-05-20 locked `Server` as the user-facing community backed by one separately runnable server runtime authority.

## Terms

### Definitions

- Server: A user-facing HexRelay community/workspace and its separately runnable authority. It may run locally, on a LAN host, privately online, or on a VPS; servers are the peers in HexRelay's server-to-server network.
- Server runtime: API/realtime/data-service process set that owns one server authority.
- Persona: A distinct user account/profile context available on the same device.
- Profile capsule: Versioned user profile payload split into signed public data and encrypted private data.
- Federation-lite: MVP discovery model using signed registry metadata rather than full decentralized DHT.
- Cutover: Optional migration action that revokes old device sessions after successful import.
- `.hxb` bundle: Encrypted and signed migration package used in full device migration.
- Shared-server discovery: User discovery mode limited to people who share at least one server context.
- Server-to-server P2P network: The dynamic policy graph formed by HexRelay server runtimes that peer with one another for server-to-server discovery, encrypted-envelope delivery, and optional relay; clients attach to servers.
- Message server: Server/runtime role in the server-to-server network that stores and forwards E2EE DM envelopes plus minimal delivery metadata, without DM plaintext or private-key custody.
- Server descriptor: Short-lived signed server metadata that advertises addresses, public keys, supported protocols, discovery policy, peering policy, relay policy, delivery policy, storage policy, rate limits, and revocation information.
- Discovery policy: Server policy controlling where a signed descriptor may appear, such as nowhere, LAN announcement, private allowlist, member-visible scope, user-consented introduction, public registry, or future public DHT.
- Peering policy: Server policy controlling which other servers may attempt authenticated server-to-server sessions.
- Relay policy: Server policy controlling whether a server forwards encrypted envelopes for other servers; relay permission never implies plaintext access.
- DM forwarding policy: Server policy controlling which encrypted DM envelope flows the server accepts for local recipients or allowlisted routes.
- User-consented server introduction: Explicit user action that shares an allowed server descriptor from one server context to another; it creates a candidate peer only and does not bypass either server's policy.
- Private online server: Server hosted on a network-reachable machine while refusing public discovery, unrestricted peering, or relay.
- Local-only server: Server runtime that refuses external discovery and peering while still serving local desktop or local server behavior.
- LAN-only server: Server runtime that may announce or accept peers on a local network but refuses WAN/public discovery.
- Policy graph: The effective server-to-server topology created by signed descriptors, authenticated peer edges, and local discovery/peering/relay/delivery/storage rules.
- E2EE DM envelope: Ciphertext payload encrypted on the sender device for recipient devices; servers/message servers may route and store it but cannot decrypt it.
- Encrypted mailbox: Bounded message-server storage for E2EE DM envelopes and delivery metadata; it must never contain server-readable DM plaintext.
- DM plaintext: Decrypted DM content and views that exist only on client/user devices.
- Nonce challenge: Server-issued one-time value signed by the client key to prove identity ownership at login.
- Forward secrecy: Property where compromise of long-term keys does not expose past DM plaintext.
- Profile replica: Server-stored copy of profile capsule data, with signed public data and encrypted private data.
- Mediated friend request: Server-routed contact request where identity bootstrap data is shared only after recipient acceptance.
- DM inbound policy: Per-user rule controlling who can start DMs (default friends-only, optional same-server or anyone).
- DM offline outbox: Client-side pending-send state used before ciphertext envelopes are durably accepted by the message server.
- Desktop local-first mode: Default runtime where the installed desktop app starts UI plus local API/realtime services for off-grid operation.
- Dedicated server mode: Optional headless deployment where API/realtime services run as a standalone server and clients connect remotely.
- Runtime components: Distinct executable boundaries (`apps/web`, `services/api-rs`, `services/realtime-rs`) that may ship in one installer but do not become one process.
- Servers Hub: App aggregation surface for joined servers. It can list several joined servers, but each server belongs behind its own server authority/endpoint.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
