# HexRelay Dependencies and Risks

## Document Metadata

- Doc ID: dependencies-risks
- Owner: Product and architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/product/04-dependencies-risks.md`

## Quick Context

- Primary edit location for dependency status and risk severity/likelihood updates.
- Record material register changes in `docs/planning/05-iteration-log.md`.
- Latest meaningful change: 2026-03-04 marked D-001 through D-007 ready with scaffold, infra, CI, crypto, and TURN/NAT artifacts.

## Purpose

- Keep one canonical dependency and risk register for MVP delivery.
- Avoid duplicated risk tracking across PRD, plan, and sprint docs.

## Dependency Register

| ID | Dependency | Type | Status | Impact if delayed | Owner | Notes |
|---|---|---|---|---|---|---|
| D-001 | Monorepo scaffold (`apps/web`, Rust services, infra) | Internal | ready | Blocks all code implementation and local bootstrap | Core | Iteration 1 baseline completed with `apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`, and `scripts` |
| D-002 | Local infra compose stack (Postgres, Redis, object storage, coturn) | Internal | ready | Blocks auth, messaging, and voice integration tests | Core | `infra/docker-compose.yml` and `infra/README.md` present |
| D-003 | CI matrix for Rust + web lint/test/build + security audit gates | Internal | ready | Increases regression, dependency, and secure-coding drift risk | Platform | Implemented in `.github/workflows/ci.yml` including cargo/npm audits, semgrep scan, and integration evidence artifact upload |
| D-004 | Runtime REST OpenAPI contract artifact (`docs/contracts/runtime-rest-v1.openapi.yaml`) | Internal | ready | Blocks API/Web parallel implementation and schema freeze enforcement | API | Required before Week 2 starts |
| D-005 | MVP Crypto Profile v1 implementation alignment | Internal | ready | Auth/E2EE tasks can diverge and fail interoperability/security tests | Core | Artifact: `docs/contracts/crypto-profile-v1.md`; checklist: `docs/testing/crypto-conformance-checklist.md` |
| D-006 | UI navigation authority mapping from spec to tasks | Internal | ready | Navigation features may be omitted or inconsistent at implementation time | Web | Trace matrix present in Iteration 2 board |
| D-007 | TURN connectivity test environment for NAT-restricted scenarios | External | ready | Voice reliability gates cannot be validated realistically | Platform/Realtime | Profile and procedure defined in `docs/planning/turn-nat-test-profile.md`; required before Iteration 3 exit |
| D-008 | Realtime event/signaling contract artifact (`docs/contracts/realtime-events-v1.asyncapi.yaml`) | Internal | ready | Realtime and web event payloads can drift and break compatibility | Realtime | Required before Iteration 2 realtime fanout sign-off |
| D-009 | Fixed KPI/SLO test profile (`docs/planning/kpi-slo-test-profile.md`) | Internal | ready | KPI/SLO evidence cannot be compared objectively across runs | Platform | Required before Iteration 4 SLO sign-off |

## Risk Register

| ID | Risk | Severity | Likelihood | Mitigation | Owner |
|---|---|---|---|---|---|
| R-001 | Scope creep in decentralization scope | high | medium | Keep MVP to federation-lite signed registry discovery | Product |
| R-002 | Voice quality instability across network conditions | high | medium | Enforce TURN fallback and add diagnostics/soak tests | Realtime |
| R-003 | E2EE implementation complexity delays messaging roadmap | high | medium | Keep 1:1 and group DM E2EE in MVP scope; reduce risk by phased delivery (1:1 baseline then group rollout in Iteration 2 with explicit test gates) | API |
| R-004 | Migration bundle integrity or restore mismatch | high | low | Versioned schemas, signed+encrypted bundles, deterministic reconcile checks, and user-signed profile precedence policy | API/Core |
| R-005 | Invite token leakage or replay | medium | medium | Hashed token storage, revoke support, one-time/TTL options for restricted servers, and monitoring for long-lived multi-use token abuse | API |
| R-006 | Key loss causing account lockout | medium | medium | Recovery phrase/device-link flow plus encrypted backup export | Product |
| R-007 | User identity scraping via discovery/friend workflows | high | medium | Enforce mediated friend requests, hide raw key/profile-identifying data by default, release bootstrap data only on accepted requests | API |
| R-008 | Missed DM delivery when recipient remains offline | medium | medium | Encrypted local outbox retries, explicit delivery-state UI, and user guidance that offline queue is best-effort in MVP | Core/Web |

## Review Cadence

- Review at each iteration start and end.
- Update severity/likelihood when evidence changes.
- Link material changes in `docs/planning/05-iteration-log.md`.

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
| R-008 | T4.5.2, T4.5.4, T4.1.2 |

## Decisions

| Decision ID | Decision | Status | Source |
|---|---|---|---|
| DEC-001 | MVP stack baseline uses Next.js + Rust + PostgreSQL + Redis + S3-compatible storage + coturn | accepted | `docs/architecture/adr-0001-stack-baseline.md` |
| DEC-002 | Task-level execution authority is owned by iteration boards, not strategy docs | accepted | `docs/product/01-mvp-plan.md` |
| DEC-003 | Profile data authority remains user-signed canonical data; server replicas are non-authoritative except server-owned security/membership fields | accepted | `docs/product/01-mvp-plan.md` |
| DEC-004 | Post-MVP discovery roadmap is hybrid: federation supported, trusted registries added, and full P2P discovery optional | accepted | `docs/product/01-mvp-plan.md` |
| DEC-005 | Server invite policy allows optional expiration/max-uses, including non-expiring multi-use links for open-access behavior | accepted | `docs/product/01-mvp-plan.md` |
| DEC-006 | Friend requests are server-mediated with privacy-first identity exposure; DM inbound defaults to friends-only with user-configurable overrides | accepted | `docs/product/01-mvp-plan.md` |
| DEC-007 | DM transport is direct user-to-user and guild/community servers do not relay/store DM payloads | accepted | `docs/product/01-mvp-plan.md` |
| DEC-008 | MVP DM offline behavior is best-effort online with encrypted local outbox retries and no guaranteed offline queue | accepted | `docs/product/01-mvp-plan.md` |

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/planning/iterations/01-sprint-board.md`
- `docs/planning/05-iteration-log.md`
