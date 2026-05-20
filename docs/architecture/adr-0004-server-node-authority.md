# ADR-0004: Server Node Authority

## Document Metadata

- Doc ID: adr-0004-server-node-authority
- Owner: Architecture maintainers
- Status: accepted
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/architecture/adr-0004-server-node-authority.md`

## Quick Context

- Primary decision authority for the relationship between a user-facing HexRelay server, server runtime, node identity, and local data boundary.
- Update this ADR when server creation, invite redemption, node identity, or server membership storage semantics change.
- Latest meaningful change: 2026-05-20 locked the user-facing server model to one separately runnable server runtime/node authority.

## Status

Accepted

## Context

HexRelay needs a decentralized/self-hostable mental model where joining or creating a server means joining or creating a real independently operated authority. The previous scaffold could also be read as many user-facing servers stored inside one API database. That shape is useful for early UI and database scaffolding, but it makes the product feel like a centralized workspace app and weakens the server/node trust boundary.

The project already has node identity, node descriptors, dedicated-server packaging, server-node DM forwarding, and app-mediated node administration. This ADR makes those pieces the canonical interpretation for user-facing servers.

## Decision

- One user-facing HexRelay server maps to one separately runnable server runtime/node authority.
- A server runtime owns one node identity/fingerprint and one node-authoritative state boundary.
- The normal user app is a client and supervisor. It may connect to local, LAN, private online, or public nodes, and it may help spawn local server runtimes, but it is not the authority for many unrelated servers inside one app database.
- If two servers run on the same physical machine, they are still separate server instances/nodes with distinct node identities, state directories, configuration, and policy.
- Server invites target a node endpoint plus node fingerprint. Redeeming an invite creates membership in that specific server node.
- The Servers Hub is an app aggregation surface over joined server nodes. It must not imply that one API runtime owns all listed servers.
- The Contacts Hub is a user/contact aggregation surface. Contacts and servers should share UX patterns where approved, but they do not share authority semantics.
- Current `servers` and `server_memberships` tables are transitional local-node persistence for the connected node's own server identity and memberships. They are not the target design for hosting many independent user-facing servers inside one API database.
- Runtime API routes that carry `server_id` must treat it as the connected node/server identity. Requests for a different server id belong to another node endpoint.

## Consequences

- Server creation from the app must create or provision a server runtime/node, not only insert a row into the current user's app database.
- Server join must bind membership to the joined node identity and endpoint.
- Multi-server desktop convenience is implemented by supervising multiple node runtimes or connecting to multiple node endpoints, each with its own state boundary.
- Tests may still use multiple `server_id` values for repository-level relational integrity checks, but API-facing membership authorization must not allow one local API runtime to impersonate many server authorities.
- Future schema cleanup should converge `server_id` and local node identity, or replace transitional server rows with explicit node-local membership state.
- Import/export and migration flows must distinguish user/app connection state from node-owned server data.
- Operator controls remain node-owned and permission-gated through authenticated app-to-node APIs.

## Alternatives Considered

- Many user-facing servers inside one API database: rejected as the canonical product model because it creates a centralized authority boundary and makes decentralization mostly cosmetic.
- One physical process hosting multiple node authorities in one database: deferred. It may be considered later only with explicit per-node identity, state partitioning, export/import, and operational isolation semantics.

## Related Documents

- `docs/architecture/01-system-overview.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/reference/glossary.md`
