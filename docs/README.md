# HexRelay Documentation Index

## Document Metadata

- Doc ID: docs-index
- Owner: HexRelay maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-03
- Source of truth: `docs/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-04-03 tightened contract-parity governance further by extending routed `500` parity to local `internal_error(...)` helper/delegate flows, with refreshed contributor/readiness operations guidance.

## Purpose

- Canonical routing for project documentation and update responsibilities.
- `Status: ready` means this index is the canonical docs router, not that every deferred runtime gap is closed; check open `watch` entries in `docs/operations/readiness-corrections-log.md` before relying on current implementation assumptions.

## Source-of-Truth Matrix

| Topic | Canonical document | Owner | Update trigger |
|---|---|---|---|
| Product intent, constraints, architecture baseline | `docs/product/01-mvp-plan.md` | Product/architecture maintainers | Scope, constraints, architecture, privacy, or security decision changes |
| Whole-system runtime topology and trust-boundary overview | `docs/architecture/01-system-overview.md` | Architecture maintainers | Runtime topology, component boundaries, trust zones, or whole-system guarantees change |
| Runtime and deployment modes (desktop local-first + dedicated server) | `docs/architecture/adr-0002-runtime-deployment-modes.md` | Architecture maintainers | Runtime packaging, deployment modes, or trust boundary behavior changes |
| Product requirements and success metrics | `docs/product/02-prd-v1.md` | Product maintainers | Functional/non-functional requirements, user flows, or success metrics change |
| Current runtime REST contract baseline | `docs/contracts/runtime-rest-v1.openapi.yaml` | API maintainers | Any implemented identity/auth/invite/friends REST schema or error change |
| Runtime REST compatibility alias (legacy filename) | `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml` | API maintainers | Keep for backward compatibility references until deprecation removal |
| MVP crypto profile contract baseline | `docs/contracts/crypto-profile-v1.md` | Core/security maintainers | Any crypto algorithm, nonce, replay, key rotation, or crypto error contract change |
| Current runtime realtime/signaling contract baseline | `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml` | Realtime maintainers | Any implemented websocket/signaling event schema change |
| Target-state MVP REST contract model | `docs/contracts/mvp-rest-v1.openapi.yaml` | API maintainers | Target endpoint/schema model changes for upcoming iterations |
| Target-state realtime event/signaling model | `docs/contracts/realtime-events-v1.asyncapi.yaml` | Realtime maintainers | Target event/schema model changes for upcoming iterations |
| MVP UI navigation and layout authority | `docs/product/07-ui-navigation-spec.md` | Product/design maintainers | Navigation paradigm, screen hierarchy, or hub behavior changes |
| MVP screen and state authority | `docs/product/08-screen-state-spec.md` | Product/design maintainers | Screen states, flow transitions, or policy-driven UI behavior changes |
| Runtime service environment/config reference | `docs/reference/runtime-config-reference.md` | Platform maintainers | `services/*/src/config.rs` or `services/*/.env.example` changes |
| Configuration defaults and override precedence | `docs/product/09-configuration-defaults-register.md` | Product/platform maintainers | Product/policy default values, ranges, or override policy changes |
| Product clarifications and open questions | `docs/product/03-clarifications.md` | Product maintainers | Any assumption is resolved, added, or materially changed |
| Infrastructure-free DM connectivity solution authority | `docs/product/10-infra-free-dm-connectivity-proposals.md` | Product/realtime maintainers | DM connectivity policy, direct-connect mechanisms, or acceptance criteria change |
| Infrastructure-free DM connectivity execution planning authority | `docs/planning/infra-free-dm-connectivity-execution-plan.md` | Delivery/core/realtime maintainers | DM connectivity sequencing, task gates, or acceptance evidence changes |
| Dependencies and risk register | `docs/product/04-dependencies-risks.md` | Product/architecture maintainers | Dependency status or risk severity/mitigation changes |
| Iteration execution planning | `docs/planning/iterations/README.md` | Delivery maintainers | Task sequencing, ownership, dependencies, or status changes |
| KPI/SLO benchmark environment profile | `docs/planning/kpi-slo-test-profile.md` | Platform maintainers | Test environment assumptions, load profile, or benchmark matrix changes |
| TURN/NAT constrained-network voice validation profile | `docs/planning/turn-nat-test-profile.md` | Platform/realtime maintainers | NAT scenario matrix, relay expectations, or constrained-network evidence contract changes |
| Iteration-level change log | `docs/planning/05-iteration-log.md` | Delivery maintainers | Scope, sequencing, status, or decision changes during execution |
| Data lifecycle and retention boundaries | `docs/architecture/02-data-lifecycle-retention-replication.md` | Architecture/API maintainers | Persistence ownership, retention, or reconciliation rules change |
| Rust service migration baseline and file mapping | `docs/architecture/03-rust-service-migration-baseline.md` | Architecture maintainers | Rust service module migration scope, baseline evidence, or mapping changes |
| Communication networking layer architecture and implementation divergence | `docs/architecture/04-communication-networking-layer-plan.md` | Architecture/core/realtime maintainers | Shared communication-layer boundaries, DM/server transport divergence, or networking rollout phases change |
| MVP operational runbook | `docs/operations/01-mvp-runbook.md` | Platform maintainers | Incident/recovery/backup procedures change |
| Dedicated-server deployment baseline | `docs/operations/02-dedicated-server-deployment.md` | Platform maintainers | Dedicated operator bring-up, ingress, remote smoke, or deployment-scope assumptions change |
| Local development prerequisites | `docs/operations/dev-prerequisites.md` | Platform maintainers | Required local tooling versions or setup flow changes |
| Migration evidence template | `docs/operations/migration-validation-template.md` | Delivery/platform maintainers | Migration evidence fields, required artifacts, or validator contract changes |
| Readiness corrections and recurrence prevention log | `docs/operations/readiness-corrections-log.md` | Maintainers | Any readiness fix lands or a previously closed finding regresses |
| MVP requirement-to-evidence verification | `docs/testing/01-mvp-verification-matrix.md` | Delivery/QA maintainers | Verification mapping, evidence format, or validator rules change |
| Observability evidence template | `docs/testing/observability-evidence-template.md` | Delivery/QA maintainers | Observability evidence format, provenance fields, or SLO reporting requirements change |
| Crypto profile conformance verification | `docs/testing/crypto-conformance-checklist.md` | QA/core maintainers | Crypto profile requirement, verification steps, or evidence expectations change |
| Contributor workflow and release hygiene | `docs/operations/contributor-guide.md` | Maintainers | Branch/PR policy, validation gates, or release workflow changes |
| Architecture decisions (ADR set) | `docs/architecture/README.md` and `docs/architecture/adr-*.md` | Architecture maintainers | Any accepted/rejected architecture decision |
| Project glossary and canonical terms | `docs/reference/glossary.md` | Product/engineering maintainers | New domain term appears or an existing term meaning changes |
| Documentation topology and ownership | `docs/README.md` | Maintainers | New docs are added, moved, or retired |

## Documentation Structure

- `docs/product/`: product-level references and navigation
- `docs/planning/`: execution planning guidance and templates
- `docs/architecture/`: architecture decision records (ADRs)
- `docs/operations/`: contributor and process operations
- `docs/reference/`: shared definitions and reference material
- `docs/testing/`: verification and evidence governance

## Lightweight Governance

- If canonical docs change, update `last_updated` and affected links in this file in the same PR.
- If docs are moved or renamed, keep compatibility stubs at old paths for at least one release cycle or two completed iterations, whichever is longer.
- Label docs PRs as either `move-only` or `content-change` in the PR body.
- Keep IDs and naming stable (`kebab-case` for docs, `README.md` for folder indexes).

## Canonical Layout

- Product docs live under `docs/product/`.
- Planning boards live under `docs/planning/iterations/`.
- Shared reference docs live under `docs/reference/`.

## Related Documents

- `README.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/product/08-screen-state-spec.md`
- `docs/product/09-configuration-defaults-register.md`
- `docs/contracts/runtime-rest-v1.openapi.yaml`
- `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
- `docs/contracts/README.md`
- `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`
- `docs/contracts/crypto-profile-v1.md`
- `docs/contracts/mvp-rest-v1.openapi.yaml`
- `docs/contracts/realtime-events-v1.asyncapi.yaml`
- `docs/architecture/02-data-lifecycle-retention-replication.md`
- `docs/architecture/01-system-overview.md`
- `docs/architecture/03-rust-service-migration-baseline.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/architecture/adr-0003-rust-service-module-architecture.md`
- `docs/operations/01-mvp-runbook.md`
- `docs/operations/02-dedicated-server-deployment.md`
- `docs/operations/contributor-guide.md`
- `docs/operations/dev-prerequisites.md`
- `docs/operations/readiness-corrections-log.md`
- `docs/reference/runtime-config-reference.md`
- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/testing/01-mvp-verification-matrix.md`
- `docs/testing/crypto-conformance-checklist.md`
- `docs/planning/05-iteration-log.md`
- `docs/operations/contributor-guide.md`
