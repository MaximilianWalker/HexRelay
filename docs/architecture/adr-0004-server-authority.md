# ADR-0004: Server Authority

## Document Metadata

- Doc ID: adr-0004-server-authority
- Owner: Architecture maintainers
- Status: accepted
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/architecture/adr-0004-server-authority.md`

## Quick Context

- Primary decision authority for the relationship between a user-facing HexRelay server, server runtime, server identity, and local data boundary.
- Update this ADR when server creation, invite redemption, server identity, or server membership storage semantics change.
- Latest meaningful change: 2026-05-20 added the destructive singleton server storage migration and removed the multi-server API database dimension.

## Status

Accepted

## Context

HexRelay needs a decentralized/self-hostable mental model where joining or creating a server means joining or creating a real independently operated authority. The previous scaffold could also be read as many user-facing servers stored inside one API database. That shape is useful for early UI and database scaffolding, but it makes the product feel like a centralized workspace app and weakens the server trust boundary.

The project already has server identity, server descriptors, dedicated-server packaging, server-to-server DM forwarding, and app-mediated server administration. This ADR makes those pieces the canonical interpretation for user-facing servers.

## Decision

- One user-facing HexRelay server maps to one separately runnable server runtime authority.
- A server runtime owns one `server_id`, one long-term server public key, and one server-authoritative state boundary.
- The normal user app is a client and supervisor. It may connect to local, LAN, private online, or public servers, and it may help spawn local server runtimes, but it is not the authority for many unrelated servers inside one app database.
- If two servers run on the same physical machine, they are still separate server authorities with distinct identities, state directories, configuration, and policy.
- Server invites target a server endpoint plus server id. Redeeming an invite creates membership in that specific server.
- The Servers Hub is an app aggregation surface over joined servers. It must not imply that one API runtime owns all listed servers.
- The Contacts Hub is a user/contact aggregation surface. Contacts and servers should share UX patterns where approved, but they do not share authority semantics.
- The API database stores one local server authority in `local_server` and server-local membership/channel/role/message tables without a `server_id` partition. API path `server_id` values identify the connected server, not a row-owned server namespace.
- Runtime API routes that carry `server_id` must treat it as the connected server identity. Requests for a different server id belong to another server endpoint.

## Consequences

- Server creation from the app must create or provision a server runtime, not only insert a row into the current user's app database.
- Server join must bind membership to the joined server identity and endpoint.
- Multi-server desktop convenience is implemented by supervising multiple server runtimes or connecting to multiple server endpoints, each with its own state boundary.
- Tests and fixtures must not seed multiple server authorities into one API database. Cross-server behavior belongs to multi-server runtime integration, not local repository fixtures.
- Import/export and migration flows must distinguish user/app connection state from server-owned server data.
- Operator controls remain server-owned and permission-gated through authenticated app-to-server APIs.

## Alternatives Considered

- Many user-facing servers inside one API database: rejected as the canonical product model because it creates a centralized authority boundary and makes decentralization mostly cosmetic.
- One physical process hosting multiple server authorities in one database: deferred. It may be considered later only with explicit per-server identity, state partitioning, export/import, and operational isolation semantics.

## Related Documents

- `docs/architecture/01-system-overview.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/reference/glossary.md`
