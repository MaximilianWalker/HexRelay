# Communication Networking Layer Plan

## Document Metadata

- Doc ID: communication-networking-layer-plan
- Owner: Architecture, core, API, and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-08
- Source of truth: `docs/architecture/04-communication-networking-layer-plan.md`

## Quick Context

- Primary edit location for networking-layer architecture across E2EE DM envelope delivery and server communication.
- Keep this plan implementation-focused and avoid duplicating product policy rationale covered in product docs.
- Latest meaningful change: 2026-05-08 locked DM networking to node/server-routed E2EE envelopes and removed user direct-DM transport/bootstrap scope.

## Purpose

- Define one networking architecture for E2EE DM envelope delivery through shared servers/message nodes plus client-to-server communication.
- Make shared policy, provenance, diagnostics, and profile-device convergence components explicit.
- Keep forbidden user direct-DM surfaces out of runtime, contracts, and product docs.

## Policy and Boundary Inputs

- DM plaintext and private keys remain client/device-only.
- Shared servers/message nodes may carry and store E2EE DM envelopes plus minimal delivery metadata only.
- Normal DM send success uses node/server encrypted-envelope delivery.
- User-to-user direct LAN/WAN DM transport, endpoint hints/cards, pairing QR/manual-code bootstrap, connectivity preflight, WAN wizard, and parallel dial are out of scope.
- Server communication remains client-to-node and can use operator-hosted server infrastructure.
- No fallback may introduce server-readable DM content, private-key custody, or unencrypted DM mailbox/relay behavior.
- One profile may run on multiple devices; incoming communication must eventually converge to all profile devices, including devices that become active later.

## Target Architecture

### Layered Model

1. **Communication Layer API**
   - Single high-level interface used by app features for DM send, channel send, and presence update.
   - Routes calls to the correct transport profile using policy rules.

2. **Session and Policy Engine**
   - Session lifecycle, auth provenance, policy checks, retry budget, and deterministic failure reasons.

3. **Delivery Diagnostics**
   - Node reachability, auth prerequisites, delivery-state diagnostics, and bounded reason codes.
   - Diagnostics must not imply peer-routed DM transport.

4. **Transport Adapters**
   - `EncryptedEnvelopeNodeTransport` for DM ciphertext envelope accept/store/fanout/catch-up.
   - `NodeClientTransport` for server/channel APIs and realtime fanout.

5. **Payload Security and Reliability**
   - DM path uses E2EE payload semantics and ciphertext-only server handling.
   - Server path uses node-authoritative channel semantics.

6. **Profile-Device Sync Layer**
   - Per-profile device manifest and device key registry.
   - Per-device ack/sync cursor tracking.
   - Active-device fanout and late-device catch-up primitives.

## Shared Responsibilities

| Capability | DM envelope path | Server communication path |
|---|---|---|
| Connection orchestration API | yes | yes |
| Session provenance model | message-node envelope acceptance provenance | node endpoint + auth provenance |
| Diagnostics reason codes | node availability, policy, retention, delivery-state, catch-up reasons | node reachability/auth reasons |
| Bootstrap format | accepted contact/friend relationship + identity/profile-device material | node endpoint and invite/auth bootstrap |
| Transport adapter | client-node ciphertext envelope store-and-forward | client-node HTTP/WebSocket |
| Message payload protection | E2EE payload required; plaintext/key custody forbidden | node policy and channel permission enforced |
| Profile-device fanout | node fanout of ciphertext envelopes to active devices | channel/presence fanout to active profile devices |
| Late-device catch-up | replay missing ciphertext envelopes by per-device cursor | replay missing channel/presence events by per-device cursor |

## Profile-Device Convergence Contract

- Profile-level requirement: all devices linked to a profile eventually converge to the same inbound communication state.
- Convergence includes devices that were offline when first delivery occurred and become active later.
- Successful DM send means durable sender-side acceptance, not merely attempted live fanout.
- Delivery model is two-phase: active-device fanout, then deferred convergence by per-device cursor and idempotent dedupe.
- Dedup identity is stable by `(message_id, profile_device_id)` for DM and `(event_id, profile_device_id)` for server-channel/presence.
- DM convergence must preserve ciphertext-only server behavior: message nodes may store/replay E2EE envelopes and minimal metadata, never plaintext or private keys.
- Live fanout failure must not discard an accepted DM; it only changes current reachability assumptions and pending delivery state.

## Scenario A: E2EE DM Delivery

### Connection Flow

1. Users establish an accepted contact/friend relationship through contact invite redemption or mediated friend request.
2. API releases only the identity and profile-device bootstrap material required for client-side E2EE setup.
3. Client encrypts message content into per-recipient/device E2EE envelopes before handing it to a shared server/message node.
4. `EncryptedEnvelopeNodeTransport` durably accepts ciphertext envelopes plus minimal delivery metadata.
5. Message node fans out ciphertext envelopes to active recipient devices and exposes per-device cursor catch-up for later-active devices.

### DM Requirements

- Sender-side success must happen only after durable acceptance of encrypted envelopes into canonical DM history plus minimal delivery metadata.
- Sender prepares per-recipient-device envelopes using recipient profile device manifest.
- Recipient devices receive ciphertext envelopes through message-node fanout and decrypt locally.
- Offline sibling devices pull missed ciphertext envelopes on activation using idempotent per-device cursor replay.
- Read-state reconciliation uses per-device cursor plus profile-level merge rule; convergence must be deterministic.
- Forbidden behavior: server-readable DM content, private-key upload/custody, server-side decryption, unencrypted DM mailboxing, plaintext relay, or user direct-DM transport/bootstrap.

## Scenario B: Server Communication

### Connection Flow

1. Client resolves node endpoint from runtime config/invite context.
2. Preflight checks node reachability, TLS expectations, and auth prerequisites.
3. `NodeClientTransport` establishes HTTP/WebSocket session to API/realtime services.
4. Session engine records auth provenance and reconnect strategy state.
5. Server/channel messaging and presence traffic use node-authoritative APIs/events.

### Server Requirements

- Node fanout targets all active devices linked to the authenticated profile.
- Later-active devices hydrate missed channel messages and presence transitions by cursor.
- Per-device channel/presence cursor state must survive reconnect and device restarts.
- DM ciphertext-only and client-only-key policy remains isolated and cannot be overridden by server transport logic.

## Communication Layer Interface Plan

### Core Interfaces

- `CommunicationMode`: `dm_envelope`, `server_channel`, `presence`.
- `TransportProfile`: selected by policy engine.
- `SessionProvenance`: mode, node endpoint, policy assertions, and auth assertions.
- `ConnectResult`: success with session metadata or deterministic failure code.
- `ProfileDeviceCursor`: per-device position for DM/server replay domains.

### Adapter Contracts

- `EncryptedEnvelopeNodeTransport`:
  - input: authenticated sender context, ciphertext envelopes, and minimal delivery metadata.
  - output: durable acceptance, active-device fanout, or deterministic delivery-state reason code.
- `NodeClientTransport`:
  - input: node endpoint and auth/session context.
  - output: node session or node-failure reason code.

### Convergence Contracts

- `DeviceManifest`: profile-linked device ids, keys, status, and revision.
- `DeliveryReceipt`: `(entity_id, profile_device_id, delivered_at)` with idempotent upsert semantics.
- `CatchUpRequest`: profile-device cursor request for missed DM/server entities.
- `CatchUpResponse`: ordered missing entities plus new cursor checkpoint.

## Delivery Ownership

- Architecture boundaries and interface contracts are defined in this document.
- Phase sequencing, task IDs, and acceptance ownership are canonical in `docs/planning/infra-free-dm-connectivity-execution-plan.md` and `docs/planning/iterations/02-sprint-board.md`.

## Validation Strategy

### Shared Layer Validation

- Contract tests for communication layer interface invariants.
- Session provenance schema tests across modes.
- Reason-code stability tests.

### DM Path Validation

- Conformance suite ensures ciphertext-only message-node handling, no server-side plaintext/private-key custody, and metadata minimization.
- Relationship bootstrap tests ensure bootstrap material is released only after accepted contact/friend state and contains no direct endpoint material.
- Active-device fanout tests ensure all online profile devices receive DM payload envelopes.
- Late-device activation tests ensure missed DM payloads converge by cursor replay.
- Guardrail tests reject user direct-DM routes, contracts, config, and runtime identifiers.

### Server Path Validation

- Node reachability/auth error taxonomy tests.
- Reconnect and ordering tests for channel and presence traffic.
- Adapter boundary tests ensuring server transport does not mutate DM ciphertext-only or client-only-key policy rules.
- Profile multi-device fanout tests ensure channel/presence events deliver to all active profile devices.
- Late-device hydration tests ensure channel/presence replay convergence by per-device cursor.

## Migration and Rollout Notes

- Keep existing non-DM feature behavior stable while tightening DM transport scope.
- Remove user direct-DM routes/contracts/UI controls before adding new node/server DM UX states.
- Block merge for DM-related PRs if policy gates detect server-readable plaintext, private-key custody, unencrypted mailbox, plaintext relay behavior, or reintroduced user direct-DM surfaces.

## Non-Goals

- Replacing node-hosted server architecture with peer-only server communication.
- Introducing plaintext/server-readable DM relay for success-rate optimization.
- Adding user direct-DM LAN/WAN transport, pairing QR/manual-code bootstrap, endpoint cards, connectivity preflight, WAN wizard, or parallel dial.
- Reworking unrelated app-layer features while implementing networking layer boundaries.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/planning/iterations/02-sprint-board.md`
