# Private Mesh Bootstrap Guide

## Document Metadata

- Doc ID: private-mesh-bootstrap
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/operations/private-mesh-bootstrap.md`

## Quick Context

- Purpose: provide the operator flow for bringing two private servers into the same signed peer mesh.
- Primary edit location: update this file when local server identity, peer invite, static peer, or private mesh bootstrap commands change.
- Latest meaningful change: 2026-05-11 added the two-server private mesh bootstrap procedure and API-to-API encrypted-envelope forwarding smoke.

## Purpose

- Show how private servers can join a small P2P mesh without public discovery.
- Keep the procedure server-to-server only; it does not introduce direct user-to-user LAN/WAN DM transport.
- Keep UX decisions out of scope. These are operator/service configuration steps only.

## Bootstrap Model

- Each server owns:
  - `API_SERVER_ID`
  - `API_LOCAL_SERVER_DESCRIPTOR_JSON`
  - `API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64`
- A server that wants another server to discover it issues a signed `PeerInviteEnvelope`.
- The recipient places that envelope inside `API_STATIC_PEER_INVITES_JSON`.
- The recipient validates the issuer descriptor, invite signature, subject server binding, TTL, discovery exposure, and invite revocation list during startup.
- Peering is directional at config time. For two servers to know each other, repeat the invite flow in both directions or combine signed invites/static descriptors as appropriate.

## Generate Server Identities

Run once per server, using stable server IDs selected by the operator.

```bash
cargo run -p api-rs --bin generate_server_identity -- --server-id server-a --address https://server-a.example --compact
cargo run -p api-rs --bin generate_server_identity -- --server-id server-b --address https://server-b.example --compact
```

The command prints:

- `api_local_server_descriptor_json`
- `api_local_server_private_key_pkcs8_base64`

Configure each server with its own output:

```text
API_SERVER_ID=server-a
API_LOCAL_SERVER_DESCRIPTOR_JSON=<server-a api_local_server_descriptor_json>
API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64=<server-a api_local_server_private_key_pkcs8_base64>
```

```text
API_SERVER_ID=server-b
API_LOCAL_SERVER_DESCRIPTOR_JSON=<server-b api_local_server_descriptor_json>
API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64=<server-b api_local_server_private_key_pkcs8_base64>
```

The generated default policy is private mesh oriented: `private_peers`, `private_allowlist`, `invite_token`, no relay, local-recipient DM forwarding, and durable encrypted envelopes.

## Issue A Signed Peer Invite

From the issuer server's environment, issue a subject-bound invite for the recipient server ID.

```bash
cargo run -p api-rs --bin issue_peer_invite -- --subject-server-id server-b --compact
```

Place the printed object as an element in the recipient's invite array:

```text
API_STATIC_PEER_INVITES_JSON=[<server-a-to-server-b PeerInviteEnvelope>]
```

For symmetric private peering, repeat from server B to server A:

```bash
cargo run -p api-rs --bin issue_peer_invite -- --subject-server-id server-a --compact
```

Then configure server A with:

```text
API_STATIC_PEER_INVITES_JSON=[<server-b-to-server-a PeerInviteEnvelope>]
```

## Validation Expectations

Startup fails when:

- the local descriptor/private key pair does not match
- the local descriptor `server_id` does not match `API_SERVER_ID`
- an invite is not signed by the issuer descriptor public key
- a subject-bound invite targets a different server ID
- the issuer descriptor refuses the invite discovery path or peering policy
- the descriptor or invite is expired or exceeds `API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS`
- the invite ID appears in `API_REVOKED_STATIC_PEER_INVITE_IDS`

The focused regression covering identity/invite bootstrap is:

```bash
cargo test -p api-rs bootstraps_two_server_private_mesh_from_generated_identity_and_signed_invite
```

The focused smoke covering actual HTTP forwarding between two local API servers is:

```bash
cargo test -p api-rs fanout_dispatch_forwards_between_two_local_api_servers_over_http
```

When running that smoke outside CI, set `API_DATABASE_URL` to the local dev Postgres URL because the path verifies durable encrypted-envelope acceptance and outbound forwarding state.

## Rotation And Revocation

- Treat `API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64` as a server-local secret. Do not commit it or share it beyond the owning server.
- Rotate a server identity by generating a new descriptor/key pair, updating that server's local env, and reissuing invites to peers that should continue discovering it.
- Revoke an issued invite by adding its invite ID to `API_REVOKED_STATIC_PEER_INVITE_IDS` on recipients that may still carry it.
- Prefer short-lived descriptors/invites for early private meshes. Reissue intentionally rather than relying on long-lived bearer material.
- If a server stops participating in the mesh, remove its descriptor/invite from peers and restart those services.

## Relay Policy Boundary

- The default generated identity does not permit relay behavior.
- Enabling relay requires explicit descriptor policy choices and should remain an operator decision.
- Even when relay is enabled later, servers carry only encrypted DM envelopes plus minimal delivery metadata; no server-readable DM plaintext or direct user-to-user DM path is introduced by this bootstrap flow.
