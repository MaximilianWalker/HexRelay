# Data Lifecycle, Retention, and Replication

## Document Metadata

- Doc ID: data-lifecycle-retention-replication
- Owner: Architecture and API maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/architecture/02-data-lifecycle-retention-replication.md`

## Quick Context

- Purpose: define where data lives, who is authoritative, and how retention/reconciliation behaves.
- Primary edit location: update when persistence boundaries or retention policy semantics change.
- Latest meaningful change: 2026-05-11 aligned DM envelope ownership with server-node P2P message-node storage and durable outbound forwarding state while keeping plaintext/private keys client-only.

## Persistence Boundary Matrix

| Entity | Authoritative owner | Stored on client | Stored on server | Retention/deletion notes |
|---|---|---|---|---|
| Identity keys | user device | yes (encrypted) | no private keys | private keys never server-persisted |
| Public profile card | user-signed state | yes | yes (replica) | newest valid signed version wins |
| Private profile capsule | user device | yes (encrypted) | yes (encrypted replica) | fail closed on signature/version mismatch |
| Friend requests | server policy + user consent | optional cache | yes | pending/accepted/declined lifecycle tracked |
| DM plaintext, decrypted views, and private keys | sender/recipient clients | yes (encrypted at rest on device) | no | never server-persisted or server-processed |
| E2EE DM envelopes | sender/recipient clients + server node/message node in the server-node P2P network | yes (encrypted local cache and decrypted local view after client decrypt) | yes (ciphertext envelopes only) | canonical encrypted DM history should not expire merely because delivery is delayed or completed; plaintext remains client-only |
| DM outbound server-node forwarding attempts | origin server node | optional cache | yes (ciphertext envelope plus minimal routing/transport state only) | retry/failure bookkeeping is origin-node state; recipient-device delivery remains ack-backed and destination-owned |
| DM per-device sync cursor/receipt state | profile devices + server node/message node in the server-node P2P network | yes | yes (minimal delivery metadata only) | used for late-device replay, idempotent dedupe, delivery-state convergence, retention, and abuse controls |
| Server channel messages | server | optional cache | yes | subject to server retention policy |
| Server channel/presence per-device cursor state | server + profile devices | yes (cache) | yes (cursor metadata) | required for late-device hydration and reconnect convergence |
| Session tokens | server | yes (session storage) | yes | revocable and expirable |
| Migration bundle metadata | migrating user | yes | no bundle plaintext | signed+encrypted bundle only |

## Retention Baseline

- Server retention defaults can be `null` (forever) or bounded by `retention.message_days`.
- Delete operations must emit deterministic tombstone semantics for sync/reconcile paths.
- Replica purge behavior follows explicit server policy and must not violate signed-profile authority model.
- DM canonical encrypted-envelope history should remain durable according to retention policy; bounded retention applies to replay acceleration state, retry bookkeeping, and transient transport metadata rather than silently discarding accepted messages.
- DM outbound server-node forwarding attempts may be retained only as long as needed for retry, diagnostics, abuse control, and reconciliation; they must not expand into plaintext storage or recipient-device endpoint tracking.
- DM plaintext and private keys must remain client/device-only regardless of server retention policy.
- Per-device cursor checkpoints must persist across restarts and support idempotent replay.
- Delivery-state metadata may be compacted once every intended recipient/device convergence rule is satisfied, but message durability is a separate concern.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/09-configuration-defaults-register.md`
