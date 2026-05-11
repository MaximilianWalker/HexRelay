# Private Mesh Bootstrap Guide

## Document Metadata

- Doc ID: private-mesh-bootstrap
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/operations/private-mesh-bootstrap.md`

## Quick Context

- Purpose: provide the operator flow for bringing two private server nodes into the same signed peer mesh.
- Primary edit location: update this file when local node identity, peer invite, static peer, or private mesh bootstrap commands change.
- Latest meaningful change: 2026-05-11 added the two-node private mesh bootstrap procedure and API-to-API encrypted-envelope forwarding smoke.

## Purpose

- Show how private server nodes can join a small P2P mesh without public discovery.
- Keep the procedure server-to-server only; it does not introduce direct user-to-user LAN/WAN DM transport.
- Keep UX decisions out of scope. These are operator/service configuration steps only.

## Bootstrap Model

- Each server node owns:
  - `API_NODE_FINGERPRINT`
  - `API_LOCAL_NODE_DESCRIPTOR_JSON`
  - `API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64`
- A server that wants another server to discover it issues a signed `PeerInviteEnvelope`.
- The recipient places that envelope inside `API_STATIC_PEER_INVITES_JSON`.
- The recipient validates the issuer descriptor, invite signature, subject node binding, TTL, discovery exposure, and invite revocation list during startup.
- Peering is directional at config time. For two nodes to know each other, repeat the invite flow in both directions or combine signed invites/static descriptors as appropriate.

## Generate Node Identities

Run once per server node, using stable node IDs selected by the operator.

```bash
cargo run -p api-rs --bin generate_node_identity -- --node-id node-a --address https://node-a.example --compact
cargo run -p api-rs --bin generate_node_identity -- --node-id node-b --address https://node-b.example --compact
```

The command prints:

- `api_local_node_descriptor_json`
- `api_local_node_private_key_pkcs8_base64`

Configure each server with its own output:

```text
API_NODE_FINGERPRINT=node-a
API_LOCAL_NODE_DESCRIPTOR_JSON=<node-a api_local_node_descriptor_json>
API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64=<node-a api_local_node_private_key_pkcs8_base64>
```

```text
API_NODE_FINGERPRINT=node-b
API_LOCAL_NODE_DESCRIPTOR_JSON=<node-b api_local_node_descriptor_json>
API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64=<node-b api_local_node_private_key_pkcs8_base64>
```

The generated default policy is private mesh oriented: `private_peers`, `private_allowlist`, `invite_token`, no relay, local-recipient DM forwarding, and durable encrypted envelopes.

## Issue A Signed Peer Invite

From the issuer node's environment, issue a subject-bound invite for the recipient node ID.

```bash
cargo run -p api-rs --bin issue_peer_invite -- --subject-node-id node-b --compact
```

Place the printed object as an element in the recipient's invite array:

```text
API_STATIC_PEER_INVITES_JSON=[<node-a-to-node-b PeerInviteEnvelope>]
```

For symmetric private peering, repeat from node B to node A:

```bash
cargo run -p api-rs --bin issue_peer_invite -- --subject-node-id node-a --compact
```

Then configure node A with:

```text
API_STATIC_PEER_INVITES_JSON=[<node-b-to-node-a PeerInviteEnvelope>]
```

## Validation Expectations

Startup fails when:

- the local descriptor/private key pair does not match
- the local descriptor `node_id` does not match `API_NODE_FINGERPRINT`
- an invite is not signed by the issuer descriptor public key
- a subject-bound invite targets a different node ID
- the issuer descriptor refuses the invite discovery path or peering policy
- the descriptor or invite is expired or exceeds `API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS`
- the invite ID appears in `API_REVOKED_STATIC_PEER_INVITE_IDS`

The focused regression covering identity/invite bootstrap is:

```bash
cargo test -p api-rs bootstraps_two_node_private_mesh_from_generated_identity_and_signed_invite
```

The focused smoke covering actual HTTP forwarding between two local API nodes is:

```bash
cargo test -p api-rs fanout_dispatch_forwards_between_two_local_api_nodes_over_http
```

When running that smoke outside CI, set `API_DATABASE_URL` to the local dev Postgres URL because the path verifies durable encrypted-envelope acceptance and outbound forwarding state.

## Rotation And Revocation

- Treat `API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64` as a server-local secret. Do not commit it or share it beyond the owning server.
- Rotate a node identity by generating a new descriptor/key pair, updating that node's local env, and reissuing invites to peers that should continue discovering it.
- Revoke an issued invite by adding its invite ID to `API_REVOKED_STATIC_PEER_INVITE_IDS` on recipients that may still carry it.
- Prefer short-lived descriptors/invites for early private meshes. Reissue intentionally rather than relying on long-lived bearer material.
- If a node stops participating in the mesh, remove its descriptor/invite from peers and restart those services.

## Relay Policy Boundary

- The default generated identity does not permit relay behavior.
- Enabling relay requires explicit descriptor policy choices and should remain an operator decision.
- Even when relay is enabled later, servers carry only encrypted DM envelopes plus minimal delivery metadata; no server-readable DM plaintext or direct user-to-user DM path is introduced by this bootstrap flow.
