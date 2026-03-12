# Communication Networking Layer Plan

## Document Metadata

- Doc ID: communication-networking-layer-plan
- Owner: Architecture, core, API, and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-12
- Source of truth: `docs/architecture/04-communication-networking-layer-plan.md`

## Quick Context

- Primary edit location for networking-layer architecture and execution across both direct DM and server communication.
- Keep this plan implementation-focused and avoid duplicating product policy rationale covered in product docs.
- Latest meaningful change: 2026-03-12 established a shared communication layer abstraction with explicit DM vs server transport divergence.

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

## Shared vs Divergent Responsibilities

| Capability | Shared | DM direct path | Server communication path |
|---|---|---|---|
| Connection orchestration API | yes | yes | yes |
| Session provenance model | yes | direct endpoint provenance | node endpoint + auth provenance |
| Diagnostics reason codes | yes | direct-connect failure reasons | node-reachability/auth reasons |
| Bootstrap format | partially | signed out-of-band pairing envelope | node endpoint and invite/auth bootstrap |
| Transport adapter | no | direct peer dial only | client-node HTTP/WebSocket |
| Message payload protection | partially | E2EE payload required | node policy and channel permission enforced |
| Fallback behavior | no | fail with guidance only | standard reconnect/retry to node |

## Scenario A: DM Direct Communication

### Connection flow

1. User imports or redeems out-of-band signed pairing envelope.
2. Pairing validation checks signature, version, expiry, replay nonce, and identity binding.
3. Preflight probes run and produce direct-connect readiness profile.
4. `DirectPeerTransport` attempts direct dial candidates only.
5. On success, session provenance records direct endpoint details and policy compliance.
6. On failure, deterministic reason code and remediation guidance are shown.

### DM-specific requirements

- Allowed transport mode is `direct_only`.
- Forbidden dependencies: STUN/TURN/relay connectivity services.
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

### Adapter contracts

- `DirectPeerTransport`:
  - input: signed pairing material + endpoint cards + policy context.
  - output: direct session or direct-failure reason code.
- `NodeClientTransport`:
  - input: node endpoint + auth/session context.
  - output: node session or node-failure reason code.

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

### Server path validation

- Node reachability/auth error taxonomy tests.
- Reconnect and ordering tests for channel and presence traffic.
- Adapter boundary tests ensuring server transport does not mutate DM policy rules.

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
