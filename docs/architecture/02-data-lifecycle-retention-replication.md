# Data Lifecycle, Retention, and Replication

## Document Metadata

- Doc ID: data-lifecycle-retention-replication
- Owner: Architecture and API maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-16
- Source of truth: `docs/architecture/02-data-lifecycle-retention-replication.md`

## Quick Context

- Purpose: define where data lives, who is authoritative, and how retention/reconciliation behaves.
- Primary edit location: update when persistence boundaries or retention policy semantics change.
- Latest meaningful change: 2026-03-16 added profile-device sync cursor and replay retention boundaries for DM and server convergence.

## Persistence Boundary Matrix

| Entity | Authoritative owner | Stored on client | Stored on server | Retention/deletion notes |
|---|---|---|---|---|
| Identity keys | user device | yes (encrypted) | no private keys | private keys never server-persisted |
| Public profile card | user-signed state | yes | yes (replica) | newest valid signed version wins |
| Private profile capsule | user device | yes (encrypted) | yes (encrypted replica) | fail closed on signature/version mismatch |
| Friend requests | server policy + user consent | optional cache | yes | pending/accepted/declined lifecycle tracked |
| DM payloads and session state | sender/recipient clients | yes (decrypted local view + encrypted local cache) | no guild server storage | direct user-to-user transport; no guild server DM relay/storage |
| DM per-device sync cursor/receipt state | profile devices | yes | optional metadata replica only (no payload content) | used for late-device replay and idempotent dedupe |
| Server channel messages | server | optional cache | yes | subject to server retention policy |
| Server channel/presence per-device cursor state | server + profile devices | yes (cache) | yes (cursor metadata) | required for late-device hydration and reconnect convergence |
| Session tokens | server | yes (session storage) | yes | revocable and expirable |
| Migration bundle metadata | migrating user | yes | no bundle plaintext | signed+encrypted bundle only |

## Retention Baseline

- Server retention defaults can be `null` (forever) or bounded by `retention.message_days`.
- Delete operations must emit deterministic tombstone semantics for sync/reconcile paths.
- Replica purge behavior follows explicit server policy and must not violate signed-profile authority model.
- DM replay metadata retention defaults to bounded window so later-active devices can converge without unbounded local growth.
- Per-device cursor checkpoints must persist across restarts and support idempotent replay.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/09-configuration-defaults-register.md`
