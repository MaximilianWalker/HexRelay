# Communication Networking Layer Plan

## Document Metadata

- Doc ID: communication-networking-layer-plan
- Owner: Architecture, core, API, and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-06
- Source of truth: `docs/architecture/04-communication-networking-layer-plan.md`

## Quick Context

- Primary edit location for networking-layer architecture and execution across both direct DM and server communication.
- Keep this plan implementation-focused and avoid duplicating product policy rationale covered in product docs.
- Latest meaningful change: 2026-04-06 clarified MVP DM reliability semantics: durable sender-side acceptance, bounded eventual catch-up, and explicit reachability downgrades without message loss.

## Purpose

- Define one networking architecture that supports both:
  - direct user-to-user DM communication, and
  - client-to-server and server-channel communication.
- Make shared components explicit so implementation avoids duplicate logic.
- Lock where the two scenarios diverge and how each path is validated.

## Policy and Boundary Inputs

- DM transport is direct user-to-user only.
- DM transport cannot depend on STUN, TURN, relay, or other always-on connectivity infrastructure.
- Server communication remains client-to-node (and optional node-to-node federation later) and can use operator-hosted server infrastructure.
- No hidden fallback may violate the DM direct-only policy.
- One profile may run on multiple devices; incoming communication must eventually converge to all profile devices, including devices that become active later.

## Target Architecture

### Layered Model

1. **Communication Layer API (shared boundary)**
   - Single high-level interface used by app features (DM send, channel send, presence update).
   - Routes calls to the correct transport profile using policy rules.

2. **Session and Policy Engine (shared)**
   - Session lifecycle (connect, handshake, rekey/refresh, close).
   - Policy checks (allowed transport mode, identity requirements, retry budget).
   - Session provenance (why/where/how connection was established).

3. **Connectivity Diagnostics (shared)**
   - Preflight probes.
   - Deterministic reason-code mapping.
   - Troubleshooter action generation.

4. **Transport Adapters (divergent)**
   - `DirectPeerTransport` for DM.
   - `NodeClientTransport` for server/channel APIs and realtime fanout.

5. **Payload Security and Reliability (partially shared)**
   - Common framing envelope and metadata.
   - DM path uses E2EE payload semantics.
   - Server path uses node-authoritative channel semantics.

6. **Profile-Device Sync Layer (shared)**
   - Per-profile device manifest and device key registry.
   - Per-device ack/sync cursor tracking.
   - Active-device fanout and late-device catch-up primitives.

## Shared vs Divergent Responsibilities

| Capability | Shared | DM direct path | Server communication path |
|---|---|---|---|
| Connection orchestration API | yes | yes | yes |
| Session provenance model | yes | direct endpoint provenance | node endpoint + auth provenance |
| Diagnostics reason codes | yes | direct-connect failure reasons | node-reachability/auth reasons |
| Bootstrap format | partially | signed out-of-band pairing envelope | node endpoint and invite/auth bootstrap |
| Transport adapter | no | direct peer dial only | client-node HTTP/WebSocket |
| Message payload protection | partially | E2EE payload required | node policy and channel permission enforced |
| Profile-device fanout | partially | recipient-device fanout + sibling replication | channel/presence fanout to all profile devices |
| Late-device catch-up | yes | replay missing DM envelopes by per-device cursor | replay missing channel/presence events by per-device cursor |
| Fallback behavior | no | fail with guidance only | standard reconnect/retry to node |

## Profile-Device Convergence Contract (Locked)

- Profile-level requirement: all devices linked to a profile eventually converge to the same inbound communication state.
- Convergence includes devices that were offline when first delivery occurred and become active later.
- Successful DM send should mean durable sender-side acceptance, not merely an attempted live fanout.
- Delivery model is two-phase:
  1. **Active-device fanout**: deliver to all currently reachable profile devices.
  2. **Deferred convergence**: replay to later-active devices using per-device cursor and idempotent dedupe.
- Dedup identity is stable by `(message_id, profile_device_id)` for DM and `(event_id, profile_device_id)` for server-channel/presence.
- DM convergence must preserve direct-only policy: no relay/server storage of DM payload content.
- Server-channel/presence convergence is node-authoritative and must hydrate all profile devices by per-device cursor.
- Live transport failure must not discard an accepted DM; it only changes current reachability assumptions and pending delivery state.
- Presence and reachability are related but distinct: a device may still appear online while current direct delivery is degraded or unreachable.

## Scenario A: DM Direct Communication

### Connection flow

1. User imports or redeems out-of-band signed pairing envelope.
2. Pairing validation checks signature, version, expiry, replay nonce, and identity binding.
3. Preflight probes run and produce direct-connect readiness profile.
4. `DirectPeerTransport` attempts direct dial candidates only.
5. On success, session provenance records direct endpoint details and policy compliance.
6. On failure, deterministic reason code and remediation guidance are shown.

### DM multi-device delivery requirements

- Sender-side success must happen only after durable acceptance into sender-controlled canonical DM history.
- Sender prepares per-recipient-device envelopes using recipient profile device manifest.
- Recipient device that first receives message records profile sync cursor and replicates missing ranges to sibling devices when reachable.
- Offline sibling devices must pull missed envelopes on activation from other profile devices using idempotent replay protocol.
- Read-state reconciliation uses per-device cursor plus profile-level merge rule; convergence must be deterministic.
- Repeated delivery failures should downgrade current reachability and suppress wasteful direct retry loops until new reachability evidence appears, but they must not erase accepted message history.

### DM-specific requirements

- Allowed transport mode is `direct_only`.
- Forbidden dependencies: STUN/TURN/relay connectivity services.
- Reliability target is durable acceptance plus bounded eventual catch-up within the decentralized model, not best-effort fire-and-forget.
- Supported reachability enhancers:
  - LAN discovery fast path (mDNS/multicast),
  - WAN direct setup wizard (UPnP/NAT-PMP/manual mapping),
  - multi-endpoint parallel dial.

## Scenario B: Server Communication

### Connection flow

1. Client resolves node endpoint from runtime config/invite context.
2. Preflight checks node reachability, TLS expectations, and auth prerequisites.
3. `NodeClientTransport` establishes HTTP/WebSocket session to API/realtime services.
4. Session engine records auth provenance and reconnect strategy state.
5. Server/channel messaging and presence traffic use node-authoritative APIs/events.

### Server multi-device delivery requirements

- Node fanout targets all active devices linked to the authenticated profile.
- Later-active devices must hydrate missed channel messages and presence transitions by cursor.
- Per-device channel/presence cursor state must survive reconnect and device restarts.

### Server-path requirements

- Standard reconnect/backoff behavior allowed.
- Node-hosted infra is expected for server communication (API/realtime runtime).
- DM direct-only policy remains isolated and cannot be overridden by server transport logic.

## Communication Layer Interface Plan

### Core interfaces

- `CommunicationMode`: `dm_direct`, `server_channel`, `presence`.
- `TransportProfile`: selected by policy engine.
- `SessionProvenance`: mode, endpoint tuple, policy assertions, auth assertions.
- `ConnectResult`: success with session metadata or deterministic failure code.
- `ProfileDeviceCursor`: per-device position for DM/server replay domains.

### Adapter contracts

- `DirectPeerTransport`:
  - input: signed pairing material + endpoint cards + policy context.
  - output: direct session or direct-failure reason code.
- `NodeClientTransport`:
  - input: node endpoint + auth/session context.
  - output: node session or node-failure reason code.

### Convergence contracts

- `DeviceManifest`: profile-linked device ids, keys, status, and revision.
- `DeliveryReceipt`: `(entity_id, profile_device_id, delivered_at)` with idempotent upsert semantics.
- `CatchUpRequest`: profile-device cursor request for missed DM/server entities.
- `CatchUpResponse`: ordered missing entities plus new cursor checkpoint.

## Delivery Ownership

- Architecture boundaries and interface contracts are defined in this document.
- Phase sequencing, task IDs, and acceptance ownership are canonical in `docs/planning/infra-free-dm-connectivity-execution-plan.md` and `docs/planning/iterations/02-sprint-board.md`.

## Validation Strategy

### Shared layer validation

- Contract tests for communication layer interface invariants.
- Session provenance schema tests across modes.
- Reason-code stability tests.

### DM path validation

- Conformance suite ensures no forbidden infra fallback calls.
- Pairing replay/expiry/signature validation tests.
- Direct-connect evidence report with deterministic failure guidance coverage.
- Active-device fanout tests ensure all online profile devices receive DM payload envelopes.
- Late-device activation tests ensure missed DM payloads converge by cursor replay.

### Server path validation

- Node reachability/auth error taxonomy tests.
- Reconnect and ordering tests for channel and presence traffic.
- Adapter boundary tests ensuring server transport does not mutate DM policy rules.
- Profile multi-device fanout tests ensure channel/presence events deliver to all active profile devices.
- Late-device hydration tests ensure channel/presence replay convergence by per-device cursor.

## Migration and Rollout Notes

- Keep existing feature behavior stable while introducing adapter boundaries.
- Migrate incrementally by routing one call-path at a time through communication layer.
- Block merge for DM-related PRs if policy gate detects forbidden fallback additions.

## Non-Goals

- Replacing node-hosted server architecture with peer-only server communication.
- Introducing infra-based DM fallback for success-rate optimization.
- Reworking unrelated app-layer features while implementing networking layer boundaries.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/planning/iterations/02-sprint-board.md`
