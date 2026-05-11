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
- Latest meaningful change: 2026-05-11 defined executable DM delivery-metadata retention and abuse-control defaults for server-node encrypted-envelope delivery while keeping plaintext/private keys client-only.

## Persistence Boundary Matrix

| Entity | Authoritative owner | Stored on client | Stored on server | Retention/deletion notes |
|---|---|---|---|---|
| Identity keys | user device | yes (encrypted) | no private keys | private keys never server-persisted |
| Public profile card | user-signed state | yes | yes (replica) | newest valid signed version wins |
| Private profile capsule | user device | yes (encrypted) | yes (encrypted replica) | fail closed on signature/version mismatch |
| Friend requests | server policy + user consent | optional cache | yes | pending/accepted/declined lifecycle tracked |
| DM plaintext, decrypted views, and private keys | sender/recipient clients | yes (encrypted at rest on device) | no | never server-persisted or server-processed |
| E2EE DM envelopes | sender/recipient clients + server node/message node in the server-node P2P network | yes (encrypted local cache and decrypted local view after client decrypt) | yes (ciphertext envelopes only) | canonical encrypted DM history should not expire merely because delivery is delayed or completed; plaintext remains client-only |
| DM outbound server-node forwarding attempts | origin server node | optional cache | yes (ciphertext envelope plus minimal routing/transport state only) | retry/failure bookkeeping, attempt count, next-attempt schedule, and terminal transport state are origin-node state; forwarded/terminal-failed metadata is purgeable after the outbound forwarding retention window |
| DM per-device sync cursor/receipt state | profile devices + server node/message node in the server-node P2P network | yes | yes (minimal delivery metadata only) | used for late-device replay, idempotent dedupe, delivery-state convergence, retention, and abuse controls; fanout delivery metadata is purgeable after the delivery-log retention window once all registered profile devices have converged, or after expiry when no profile device is registered |
| Server channel messages | server | optional cache | yes | subject to server retention policy |
| Server channel/presence per-device cursor state | server + profile devices | yes (cache) | yes (cursor metadata) | required for late-device hydration and reconnect convergence |
| Session tokens | server | yes (session storage) | yes | revocable and expirable |
| Migration bundle metadata | migrating user | yes | no bundle plaintext | signed+encrypted bundle only |

## Retention Baseline

- Server retention defaults can be `null` (forever) or bounded by `retention.message_days`.
- Delete operations must emit deterministic tombstone semantics for sync/reconcile paths.
- Replica purge behavior follows explicit server policy and must not violate signed-profile authority model.
- DM canonical encrypted-envelope history should remain durable according to retention policy; bounded retention applies to replay acceleration state, retry bookkeeping, and transient transport metadata rather than silently discarding accepted messages.
- DM outbound server-node forwarding attempts may be retained only as long as needed for retry, diagnostics, abuse control, and reconciliation; retry scheduling uses bounded attempt counts plus exponential backoff with stable jitter, and must not expand into plaintext storage or recipient-device endpoint tracking.
- DM plaintext and private keys must remain client/device-only regardless of server retention policy.
- Per-device cursor checkpoints must persist across restarts and support idempotent replay.
- Delivery-state metadata may be compacted once every intended recipient/device convergence rule is satisfied, but message durability is a separate concern.

## DM Delivery Metadata Retention

- Default fanout delivery-log retention is 30 days (`API_DM_DELIVERY_LOG_RETENTION_SECONDS=2592000`).
- Default outbound forwarding-log retention is 7 days (`API_DM_OUTBOUND_FORWARDING_LOG_RETENTION_SECONDS=604800`).
- Fanout delivery-log purge deletes only expired metadata rows that are no longer needed by registered profile devices, or expired metadata for identities with no registered profile devices.
- Canonical encrypted DM history in `dm_messages` is not deleted by delivery-metadata retention; it remains governed by the message-retention policy.
- Outbound forwarding purge deletes expired `forwarded` rows and terminal `failed` rows with no future retry schedule. Queued or retry-scheduled rows remain until retry resolution or later expiry.
- Delivery metadata retention stores no plaintext, private keys, recipient-device endpoint hints, LAN/WAN addresses, pairing payloads, or direct-transport state.
- Abuse controls for DM delivery are identity/node scoped rate limits over dispatch, catch-up, ack, and authenticated node-forward ingress; they operate on request counts and policy state, not plaintext inspection.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/product/09-configuration-defaults-register.md`
