# HexRelay Dependencies and Risks

## Document Metadata

- Doc ID: dependencies-risks
- Owner: Product and architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/product/04-dependencies-risks.md`

## Quick Context

- Primary edit location for dependency status and risk severity/likelihood updates.
- Record material register changes in `docs/planning/05-iteration-log.md`.
- Latest meaningful change: 2026-05-20 added the accepted server-node authority decision and related schema-transition risk.

## Purpose

- Keep one canonical dependency and risk register for MVP delivery.
- Avoid duplicated risk tracking across PRD, plan, and sprint docs.

## Dependency Register

| ID | Dependency | Type | Status | Impact if delayed | Owner | Notes |
|---|---|---|---|---|---|---|
| D-001 | Monorepo scaffold (`apps/web`, Rust services, infra) | Internal | ready | Blocks all code implementation and local bootstrap | Core | Iteration 1 baseline completed with `apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`, and `scripts` |
| D-002 | Local infra compose stack (Postgres, Redis, object storage, coturn) | Internal | ready | Blocks auth, messaging, and voice integration tests | Core | `infra/docker-compose.yml` and `infra/README.md` present |
| D-003 | CI matrix for Rust + web lint/test/build + security audit gates | Internal | ready | Increases regression, dependency, and secure-coding drift risk | Platform | Implemented in `.github/workflows/ci.yml` including cargo/npm audits, semgrep scan, and integration evidence artifact upload |
| D-004 | Runtime REST OpenAPI contract artifact (`docs/contracts/runtime-rest.openapi.yaml`) | Internal | ready | Blocks API/Web parallel implementation and schema freeze enforcement | API | Required before Week 2 starts |
| D-005 | MVP Crypto Profile implementation alignment | Internal | ready | Auth/E2EE tasks can diverge and fail interoperability/security tests | Core | Artifact: `docs/contracts/crypto-profile.md`; checklist: `docs/testing/crypto-conformance-checklist.md` |
| D-006 | UI navigation authority mapping from spec to tasks | Internal | ready | Navigation features may be omitted or inconsistent at implementation time | Web | Trace matrix present in Iteration 2 board |
| D-007 | E2EE DM envelope delivery conformance profile | Internal | ready | DM baseline could drift into server-readable payloads, excess metadata, private-key custody, or retired node-bypassing client DM surfaces without repeatable evidence | Core/API/QA | Conformance must prove ciphertext-only server-node/message-node handling, client-only plaintext/private keys, deterministic delivery-metadata retention, metadata-only abuse controls, and absence of node-bypassing client DM transport/bootstrap surfaces |
| D-010 | TURN/NAT constrained-network validation profile for Iteration 3 voice/screen-share flows | Internal | ready | Voice/screen-share constrained-network behavior cannot be signed off with repeatable evidence | Platform/Realtime | Canonical profile: `docs/planning/turn-nat-test-profile.md`; scoped to voice/screen-share only |
| D-008 | Realtime event/signaling contract artifact (`docs/contracts/realtime-events.asyncapi.yaml`) | Internal | ready | Realtime and web event payloads can drift and break compatibility | Realtime | Required before Iteration 2 realtime fanout sign-off |
| D-009 | Fixed KPI/SLO test profile (`docs/planning/kpi-slo-test-profile.md`) | Internal | ready | KPI/SLO evidence cannot be compared objectively across runs | Platform | Required before Iteration 4 SLO sign-off |

## Risk Register

| ID | Risk | Severity | Likelihood | Mitigation | Owner |
|---|---|---|---|---|---|
| R-001 | Scope creep in decentralization scope | high | medium | Keep MVP to federation-lite signed registry discovery plus explicit server-node policy boundaries; defer Kademlia/HyParView/Plumtree until public opt-in networks need them | Product |
| R-002 | Voice/screen-share connection failure across NAT-restricted network conditions | high | high | Enforce Iteration 3 TURN/NAT constrained-network profile (`NAT-01`..`NAT-04`), keep relay fallback success thresholds, and block iteration sign-off until profile passes | Realtime |
| R-003 | E2EE implementation complexity delays messaging roadmap | high | low | Keep 1:1 and group DM E2EE in MVP scope; current Iteration 2 evidence covers 1:1 bootstrap/rotation, group bootstrap/rekey, group ciphertext encrypt/decrypt, and missing-key recovery paths with explicit test gates | API |
| R-004 | Migration bundle integrity or restore mismatch | high | low | Versioned schemas, signed+encrypted bundles, deterministic reconcile checks, and user-signed profile precedence policy | API/Core |
| R-005 | Invite token leakage or replay | medium | medium | Hashed token storage, revoke support, one-time/TTL options for restricted servers, and monitoring for long-lived multi-use token abuse | API |
| R-006 | Key loss causing account lockout | medium | medium | Recovery phrase/device-link flow plus encrypted backup export | Product |
| R-007 | User identity or private-node scraping via discovery/friend workflows | high | medium | Enforce mediated friend requests, signed opt-in node descriptors, descriptor-scope validation, rate limits, denylists, and bootstrap release only on accepted requests | API |
| R-008 | DM encrypted-envelope delivery leaks plaintext, private keys, or excess metadata through server nodes/message nodes | high | medium | Enforce ciphertext-only schemas, client-only decryption/key storage, minimal delivery metadata, 30-day fanout metadata retention, 7-day outbound forwarding metadata retention, metadata-only rate limits, and CI guardrails that reject plaintext mailbox/relay semantics | Core/API/Security |
| R-009 | Multi-device divergence where one profile device misses messages/events after delayed activation | high | medium | Enforce per-device cursor tracking, idempotent replay/dedupe contracts, active+late-device convergence tests, and backend realtime target summaries for queued/pending device outcomes | Core/Realtime |
| R-010 | Transitional `servers`/`server_memberships` storage is mistaken for many independent servers inside one API runtime | high | medium | Treat one user-facing server as one node authority, scope API authorization to the connected node fingerprint, and schedule schema cleanup to converge server identity with node identity before Create/Join Server runtime expansion | Architecture/API |

## Review Cadence

- Review at each iteration start and end.
- Update severity/likelihood when evidence changes.
- Link material changes in `docs/planning/05-iteration-log.md`.
- Last reviewed: 2026-05-20 (added server-node authority decision and schema-transition risk).

## Risk to Task Mitigation Matrix

| Risk ID | Mitigating task IDs |
|---|---|
| R-001 | T7.2.1, T7.2.2, T7.3.1 |
| R-002 | T5.1.2, T5.2.1, T5.3.1 |
| R-003 | T4.5.1, T4.5.2, T4.5.3, T4.5.4 |
| R-004 | T7.1.2, T7.5.1, T7.5.2, T7.5.3 |
| R-005 | T2.2.1, T2.3.1, T2.4.1 |
| R-006 | T2.1.4, T7.5.2, T8.3.1 |
| R-007 | T3.1.1, T3.1.2, T3.1.5, T4.1.2 |
| R-008 | T4.1.3, T4.1.7, T4.1.8, T4.5.2, T4.5.4 |
| R-010 | T4.2.1, T4.2.2, T4.6.1 |

## Decisions

| Decision ID | Decision | Status | Source |
|---|---|---|---|
| DEC-001 | MVP stack baseline uses Next.js + Rust + PostgreSQL + Redis + S3-compatible storage + coturn | accepted | `docs/architecture/adr-0001-stack-baseline.md` |
| DEC-002 | Task-level execution authority is owned by iteration boards, not strategy docs | accepted | `docs/product/01-mvp-plan.md` |
| DEC-003 | Profile data authority remains user-signed canonical data; server replicas are non-authoritative except server-owned security/membership fields | accepted | `docs/product/01-mvp-plan.md` |
| DEC-004 | Post-MVP discovery roadmap is hybrid: private/trusted discovery first, federation registries supported, user-consented node introductions allowed by descriptor policy, and decentralized server/node discovery optional | accepted | `docs/product/01-mvp-plan.md` |
| DEC-005 | Server invite policy allows optional expiration/max-uses, including non-expiring multi-use links for open-access behavior | accepted | `docs/product/01-mvp-plan.md` |
| DEC-006 | Friend requests are server-mediated with privacy-first identity exposure; DM inbound defaults to friends-only with user-configurable overrides | accepted | `docs/product/01-mvp-plan.md` |
| DEC-007 | DM delivery baseline uses server nodes/message nodes in the server-node P2P network for E2EE envelope store-and-forward; DM plaintext and private keys remain client/device-only | accepted | `docs/product/01-mvp-plan.md` |
| DEC-008 | MVP DM offline behavior requires durable encrypted-envelope acceptance into canonical DM history plus bounded eventual catch-up | accepted | `docs/product/01-mvp-plan.md` |
| DEC-009 | Recipient-device LAN/WAN transport, pairing QR/manual-code bootstrap, endpoint hints/cards, preflight, WAN wizard, and parallel dial are out of MVP DM delivery scope | accepted | `docs/product/03-clarifications.md` |
| DEC-010 | Incoming communication must converge across all profile-linked devices (active fanout + late-device catch-up) for DM and server communication domains | accepted | `docs/product/01-mvp-plan.md` |
| DEC-011 | Server-node P2P topology is a dynamic policy graph with no primary-server assumption; discovery, peering, relay, delivery, and storage permissions are separate | accepted | `docs/architecture/04-communication-networking-layer-plan.md` |
| DEC-012 | DM delivery metadata retention is separate from canonical encrypted DM history; abuse controls are sender/device/node scoped and do not inspect plaintext | accepted | `docs/product/01-mvp-plan.md` |
| DEC-013 | One user-facing server maps to one separately runnable server runtime/node authority; multi-server-in-one-API storage is transitional only | accepted | `docs/architecture/adr-0004-server-node-authority.md` |

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/planning/iterations/01-sprint-board.md`
- `docs/planning/05-iteration-log.md`
