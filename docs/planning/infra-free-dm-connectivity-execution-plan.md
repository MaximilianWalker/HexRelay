# Infrastructure-Free DM Connectivity Execution Plan

## Document Metadata

- Doc ID: infra-free-dm-connectivity-execution-plan
- Owner: Delivery, core, and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-16
- Source of truth: `docs/planning/infra-free-dm-connectivity-execution-plan.md`

## Quick Context

- Primary edit location for phased execution of infrastructure-free DM connectivity.
- Update this plan when direct-connect task sequencing, acceptance criteria, or risk controls change.
- Cross-scenario networking architecture authority lives in `docs/architecture/04-communication-networking-layer-plan.md`.
- Latest meaningful change: 2026-03-16 expanded execution plan to include profile-device eventual-sync convergence for active and later-active devices.

## Purpose

- Translate direct-connect policy into executable backlog slices.
- Provide deterministic implementation order for DM connectivity features.
- Keep delivery aligned with product constraints and test evidence requirements.
- Avoid duplicating shared networking-layer design details that are canonical in the architecture plan.

## Locked Policy Inputs

- DM transport is direct user-to-user only.
- No STUN, TURN, or relay infrastructure dependency is allowed for DM connectivity.
- No hidden fallback violating direct-only policy is allowed.
- When direct connectivity is unavailable, the product must fail with explicit user guidance.

## Execution Phases

### Phase A: Policy and Transport Hardening

- Task IDs: `T4.1.3`
- Outcome: runtime guarantees direct-only DM transport with CI-level policy guardrails.
- Deliverables:
  - direct-only transport mode enforcement
  - policy lints/checks rejecting infra connectivity reintroduction
  - session-level direct-path provenance output

### Phase B: Trust Bootstrap Without Infrastructure

- Task IDs: `T4.1.4`
- Outcome: users can establish direct contacts via signed out-of-band pairing.
- Deliverables:
  - signed pairing envelope schema/versioning
  - QR and short-code import/export UX
  - replay/expiry/fingerprint validation

### Phase C: Deterministic Failure Handling

- Task IDs: `T4.1.5`
- Outcome: connection attempts produce stable diagnostics and guided remediation.
- Deliverables:
  - connectivity preflight probes
  - reason-code taxonomy
  - in-product troubleshooter actions

### Phase D: Reachability Improvements Without Infra

- Task IDs: `T4.1.6`, `T4.1.7`, `T4.1.8`, `T4.1.9`, `T4.1.10`
- Outcome: improved direct-connect success rates plus profile-device convergence through active fanout and late-device catch-up.
- Deliverables:
  - LAN mDNS/multicast discovery fast path
  - WAN setup wizard (UPnP/NAT-PMP attempt + manual mapping guidance)
  - multi-endpoint parallel dial support
  - active profile-device DM fanout once one recipient device is reachable
  - late-device DM replay/catch-up using deterministic per-device cursors

## Detailed Task Plan

| Task ID | Task | Owner | Depends on | Acceptance criteria |
|---|---|---|---|---|
| T4.1.3 | Enforce direct-only DM transport and infra-policy CI gate | Core | T4.1.1 | DM connect path uses direct dial only; CI rejects forbidden infra fallbacks/configs |
| T4.1.4 | Implement signed out-of-band pairing envelope + QR/short-code UX | Core/Web | T3.1.4, T4.1.3 | Pairing works without backend rendezvous; replay/expiry checks are enforced |
| T4.1.5 | Add DM connectivity preflight and deterministic troubleshooter | Core/Web | T4.1.4 | Failed connections emit stable reason codes and actionable remediation |
| T4.1.6 | Add LAN discovery fast path (mDNS/multicast, local-first dialing) | Realtime/Core | T4.1.5 | Same-LAN peers discover/connect with local-only traffic and improved latency |
| T4.1.7 | Add WAN direct-connect setup wizard (UPnP/NAT-PMP + manual) | Core/Web | T4.1.5 | Wizard produces deterministic outcomes: success/manual_required/network_incompatible |
| T4.1.8 | Add multi-endpoint cards and parallel dial orchestration | Core | T4.1.4, T4.1.6 | Parallel dial improves connect success and cancels non-winning attempts safely |
| T4.1.9 | Add DM active-device fanout for profile-linked devices | Core/Realtime | T4.1.8 | Incoming DM payload reaches all currently active devices linked to recipient profile |
| T4.1.10 | Add DM late-device catch-up by per-device cursor and dedupe | Core | T4.1.9 | Devices activated after initial receive replay missing DM payloads and converge deterministically |

## Validation and Evidence Plan

- `T4.1.3`: policy gate report + direct transport integration tests.
- `T4.1.4`: pairing conformance report (signature/expiry/replay checks) + UX state screenshots.
- `T4.1.5`: reason-code matrix report and remediation action coverage.
- `T4.1.6`: LAN discovery/connect benchmarks with local-subnet-only traffic proof and explicit confirmation that discovery hints stay ephemeral/TTL-scoped rather than DB-persisted.
- `T4.1.7`: WAN wizard scenario matrix with deterministic result classification.
- Broad off-LAN discovery is not a separate MVP feature; existing off-LAN bootstrap/pathing is pairing + WAN guidance + endpoint cards + parallel dial. Any future extension should be limited to authorized endpoint-card freshness for already-paired peers.
- Profile-device convergence is already represented by active-device fanout plus later-active catch-up. Any future extension should be limited to self/profile device-state UX or authorized endpoint-card freshness, not broad device discovery semantics.
- Contact-aware device discovery is also out of current MVP scope; if revisited, it must remain contact-authorized, pull-based, and limited to endpoint-card freshness metadata rather than friend-visible online/device presence.
- Broad multi-device DM convergence is already covered by active-device fanout, per-device cursor persistence, and later-active catch-up. The only remaining follow-up is whether replay payload durability should stay memory-bounded, move to peer-only sibling-device sync, or be explicitly approved as bounded encrypted server-side storage.
- Durable DM history should be treated separately from replay backlog durability: the intended direction is client-local or local-runtime-backed thread/message persistence, while server-side DM state remains metadata-only under the direct-only policy.
- Recipient-targeted realtime signaling should also stay a separate routing track: it is a narrow authenticated websocket signaling feature for live call setup, not part of broad DM convergence or discovery scope.
- `T4.1.8`: multi-endpoint race/connect report and endpoint revocation tests.
- `T4.1.9`: DM active-device fanout matrix for multi-device profiles.
- `T4.1.10`: late-device replay/catch-up convergence report (offline-then-activate scenarios).

Evidence path baseline:

- `evidence/iteration-02/dm-connectivity/`

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| NAT-restricted networks still fail direct connect | High | Phase C diagnostics + Phase D WAN wizard and explicit incompatibility state |
| UX complexity increases setup friction | Medium | Keep flows deterministic, with bounded state machine and clear action language |
| Policy drift reintroduces forbidden fallback | High | CI policy gate and contract-level non-goal tests in every DM transport PR |
| Multi-device convergence drift across profile devices | High | Per-device cursor contracts, idempotent replay semantics, and late-activation convergence tests |

## Non-Goals

- Adding STUN/TURN/relay for DM transport.
- Masking connection failures with hidden infra-assisted fallbacks.
- Broad architecture refactors outside DM connectivity scope.

## Related Documents

- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/testing/01-mvp-verification-matrix.md`
