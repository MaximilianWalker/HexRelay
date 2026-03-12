# Infrastructure-Free DM Connectivity Proposals

## Document Metadata

- Doc ID: infra-free-dm-connectivity-proposals
- Owner: Product and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-12
- Source of truth: `docs/product/10-infra-free-dm-connectivity-proposals.md`

## Quick Context

- Primary edit location for detailed DM connectivity solution candidates under the no-infrastructure rule.
- Update this file when direct-connect architecture, feasibility assumptions, or acceptance criteria change.
- Cross-scenario networking implementation details are canonical in `docs/architecture/04-communication-networking-layer-plan.md`.
- Latest meaningful change: 2026-03-12 established ranked implementation propositions for infrastructure-free direct DM connectivity.

## Purpose

- Convert the infrastructure-free DM policy into concrete implementation options.
- Define option trade-offs and DM-specific behavior candidates.
- Keep execution sequencing in `docs/planning/infra-free-dm-connectivity-execution-plan.md` and avoid duplicating full implementation architecture here.

## Locked Constraints

- DM transport must remain direct user-to-user.
- Solutions that require connectivity infrastructure are out of scope for DM transport.
- No STUN, TURN, or relay dependency is allowed for DM connectivity.
- When direct connectivity cannot be established, the product must fail explicitly with actionable user guidance.

## Evaluation Criteria

- **Policy compliance**: no infra dependency introduced by design or fallback.
- **Connect success rate**: measurable improvement without violating constraints.
- **User effort**: number of manual steps to establish first DM session.
- **Deterministic diagnostics**: each failure maps to a stable reason code and remediation.
- **Implementation risk**: complexity and portability across target desktop environments.

## Proposition 1 (Rank 1): Direct-Only Transport Core and Policy Guardrails

### What changes

- Remove and block relay-oriented code paths/configuration from DM transport execution.
- Expose direct transport provenance in runtime state and UI.
- Add CI policy checks that reject infra fallback reintroduction.

### How it works

1. User initiates DM connection.
2. Transport layer attempts direct endpoint dialing only (no service candidate gathering).
3. Session establishes if direct path succeeds; session metadata marks `direct=true` and stores endpoint provenance.
4. If all direct attempts fail, connection ends with explicit reason code and guided actions.
5. CI/build-time policy checks fail if forbidden infra keys or fallback callsites are added.

### Trade-offs and risks

- Lower success rate under restrictive NAT/firewall conditions.
- Higher pressure on diagnostics and setup UX quality.
- Requires strict governance to prevent accidental policy drift.

### Acceptance criteria

- Integration tests show zero STUN/TURN/relay network calls during DM setup.
- Session details expose direct-path provenance for 100% successful DM sessions.
- CI policy gate fails on introduction of banned infra connectivity flags/paths.

### Implementation slices

- Remove infra-related runtime flags and fallback paths from DM transport code.
- Add direct transport provenance object and API/UI surfacing.
- Add static and runtime guard tests for policy enforcement.

## Proposition 2 (Rank 2): Out-of-Band Pairing Envelope (QR + Short Code)

### What changes

- Replace service-based rendezvous with signed out-of-band pairing artifacts.
- Add QR flow and short-code fallback for endpoint and identity exchange.

### How it works

1. Sender creates a signed pairing envelope containing identity key, endpoint hints, nonce, and expiry.
2. Envelope is shared directly (QR scan, local file transfer, or short code transcription).
3. Receiver validates signature, expiry, version, and replay nonce.
4. On success, both clients store peer identity and endpoint cards for direct dialing.
5. On failure (expired/replayed/corrupt), UX shows deterministic reason and regeneration action.

### Trade-offs and risks

- More steps than service-mediated invites.
- Endpoint hints can become stale.
- Requires careful UX for non-technical users.

### Acceptance criteria

- Pairing flow runs without backend rendezvous dependency.
- Replayed and expired envelopes are rejected in all security tests.
- Median time-to-pair remains within target usability budget.

### Implementation slices

- Define envelope schema/versioning and signing contract.
- Implement QR encoding/decoding plus short-code codec.
- Add replay/expiry protection and guided recovery UI.

## Proposition 3 (Rank 3): Connectivity Preflight and Deterministic Troubleshooter

### What changes

- Add pre-connect checks and a fixed failure taxonomy.
- Provide guided remediation before and after failed direct attempts.

### How it works

1. User clicks "Start DM" and preflight runs local checks (port bind ability, local interface state, known blocking indicators).
2. Client scores readiness and selects direct dial strategy.
3. On failure, system maps event to a bounded reason code set (for example: `port_blocked`, `peer_unreachable`, `nat_restricted`).
4. UI shows exact next actions (retry window, local firewall exception, router mapping guidance, local network alternative).
5. User can re-run preflight and compare outcomes.

### Trade-offs and risks

- False diagnostics can hurt trust.
- Platform-specific network behaviors require per-OS tuning.

### Acceptance criteria

- Every failed direct attempt emits one stable reason code.
- Troubleshooter provides at least one concrete remediation action for each reason code.
- First-attempt failure rate decreases measurably after rollout.

### Implementation slices

- Implement preflight probe module.
- Define reason-code contract and UI mapping.
- Add retry loop with before/after diagnostics snapshot.

## Proposition 4 (Rank 4): Local-Network Fast Path (mDNS/Multicast + Direct QUIC/TCP)

### What changes

- Prioritize same-LAN discovery and direct connect path.
- Reduce initial friction for users on shared/home/local networks.

### How it works

1. Client advertises presence on local subnet via mDNS/multicast.
2. Peer discovery exchanges signed endpoint cards locally.
3. Transport attempts direct connection over local prioritized endpoints first.
4. If local discovery fails, user falls back to out-of-band pairing from Proposition 2.

### Trade-offs and risks

- Discovery may be blocked on enterprise or segmented networks.
- Additional platform networking code paths increase maintenance.

### Acceptance criteria

- High same-LAN discovery and connect success in controlled test matrix.
- Discovery traffic never leaves local subnet.
- Local-path median connect latency beats non-local path baseline.

### Implementation slices

- Add mDNS advertisement and discovery service.
- Add multicast fallback and network capability detection.
- Add local-first dial prioritization and test harness coverage.

## Proposition 5 (Rank 5): WAN Direct Connectivity Wizard (UPnP/NAT-PMP + Manual Mapping)

### What changes

- Add guided setup for direct WAN reachability without relay infrastructure.
- Use automation first, then deterministic manual guidance.

### How it works

1. Wizard attempts automatic router mapping with UPnP/NAT-PMP.
2. Client validates reachable port state from user-provided peer test.
3. If automation fails, UI shows deterministic manual mapping steps and verification checklist.
4. User reruns validation until direct path success or explicit "not possible on this network" outcome.

### Trade-offs and risks

- Router behaviors vary widely; success cannot be guaranteed.
- Some networks remain fundamentally incompatible with direct inbound flows.

### Acceptance criteria

- Home-network matrix produces target minimum WAN direct success.
- Wizard emits deterministic outcomes: `success`, `manual_required`, `network_incompatible`.
- Manual path documentation is validated against representative routers.

### Implementation slices

- Implement UPnP/NAT-PMP attempt module with bounded timeouts.
- Implement reachable-port validation routine and result contract.
- Implement manual wizard UI and copyable diagnostics packet.

## Proposition 6 (Rank 6): Multi-Endpoint Parallel Dial (User-Owned Devices Only)

### What changes

- Allow each user to publish multiple direct endpoint cards (desktop/laptop/phone).
- Improve success rates by racing direct attempts across valid endpoints.

### How it works

1. Pairing envelope carries multiple endpoint cards with expiry and signing metadata.
2. Initiator launches parallel direct dial attempts with deterministic concurrency limits.
3. First successful direct session wins; remaining attempts are canceled.
4. Endpoint health stats update locally to prioritize better cards in subsequent sessions.

### Trade-offs and risks

- More endpoint lifecycle complexity (expiry, revocation, stale cards).
- Slight additional power/network cost during connect windows.

### Acceptance criteria

- Connection success improves versus single-endpoint mode in controlled test profile.
- Endpoint revocation blocks stale endpoint usage deterministically.
- Parallel dial limits prevent resource exhaustion under repeated retries.

### Implementation slices

- Extend pairing envelope to include endpoint card arrays.
- Implement parallel dial orchestration and winner selection.
- Add endpoint management UI and revocation handling.

## Delivery Ownership

- This document is the option catalog and trade-off authority.
- Sequencing and task ownership are canonical in `docs/planning/infra-free-dm-connectivity-execution-plan.md`.

## Non-Goals

- No infrastructure-assisted NAT traversal for DM transport.
- No guild/community server relay path for DM payload delivery.
- No hidden fallback that violates policy while preserving apparent UX success.

## Related Documents

- `AGENTS.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/planning/05-iteration-log.md`
