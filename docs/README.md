# HexRelay Documentation Index

## Document Metadata

- Doc ID: docs-index
- Owner: HexRelay maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-15
- Source of truth: `docs/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-05-15 documented JSON request media-type contract-parity coverage.

## Purpose

- Canonical routing for project documentation and update responsibilities.
- `Status: ready` means this index is the canonical docs router, not that every deferred runtime gap is closed; check open `watch` entries in `docs/operations/readiness-corrections-log.md` before relying on current implementation assumptions.

## Source-of-Truth Matrix

| Topic | Canonical document | Owner | Update trigger |
|---|---|---|---|
| Product intent, scope, and non-architectural constraints | `docs/product/01-mvp-plan.md` | Product/architecture maintainers | Scope, product constraints, privacy, or security decision changes |
| Architecture baseline and whole-system runtime design authority | `docs/architecture/01-system-overview.md`, `docs/architecture/02-data-lifecycle-retention-replication.md`, `docs/architecture/04-communication-networking-layer-plan.md`, and relevant `docs/architecture/adr-*.md` | Architecture maintainers | Runtime boundaries, architecture baseline, trust zones, persistence ownership, or accepted design decisions change |
| Whole-system runtime topology and trust-boundary overview | `docs/architecture/01-system-overview.md` | Architecture maintainers | Runtime topology, component boundaries, trust zones, or whole-system guarantees change |
| Runtime and deployment modes (desktop local-first + dedicated server) | `docs/architecture/adr-0002-runtime-deployment-modes.md` | Architecture maintainers | Runtime packaging, deployment modes, administration surface, or trust boundary behavior changes |
| Product requirements and success metrics | `docs/product/02-prd.md` | Product maintainers | Functional/non-functional requirements, user flows, or success metrics change |
| Current runtime REST contract baseline | `docs/contracts/runtime-rest.openapi.yaml` | API maintainers | Any implemented REST endpoint, schema, auth behavior, or error change |
| MVP crypto profile contract baseline | `docs/contracts/crypto-profile.md` | Core/security maintainers | Any crypto algorithm, nonce, replay, key rotation, or crypto error contract change |
| Current runtime realtime/signaling contract baseline | `docs/contracts/realtime-events-runtime.asyncapi.yaml` | Realtime maintainers | Any implemented websocket/signaling event schema change |
| Target-state MVP REST contract model | `docs/contracts/mvp-rest.openapi.yaml` | API maintainers | Target endpoint/schema model changes for upcoming iterations |
| Target-state realtime event/signaling model | `docs/contracts/realtime-events.asyncapi.yaml` | Realtime maintainers | Target event/schema model changes for upcoming iterations |
| MVP UI navigation and layout authority | `docs/product/07-ui-navigation-spec.md` | Product/design maintainers | Navigation paradigm, screen hierarchy, or hub behavior changes |
| Navigation implementation planning and approval package | `docs/planning/navigation-implementation-plan.md` | Web and delivery maintainers | `T4.6.1`-`T4.6.4` sequencing, task slicing, approval package, or navigation evidence plan changes |
| MVP screen and state authority | `docs/product/08-screen-state-spec.md` | Product/design maintainers | Screen states, flow transitions, or policy-driven UI behavior changes |
| Runtime service environment/config reference | `docs/reference/runtime-config-reference.md` | Platform maintainers | `services/*/src/config.rs`, `services/*/.env.example`, or runtime env semantics change |
| Local runtime testing operational quickstart | `docs/operations/local-runtime-testing-quickstart.md` | Platform/QA maintainers | Local fixture, runtime-profile, Docker runtime, network simulation, or troubleshooting workflow changes |
| Configuration defaults and override precedence | `docs/product/09-configuration-defaults-register.md` | Product/platform maintainers | Product/policy default values, ranges, or override policy changes |
| Product clarifications and open questions | `docs/product/03-clarifications.md` | Product maintainers | Any assumption is resolved, added, or materially changed |
| DM encrypted-envelope delivery solution authority | `docs/product/10-infra-free-dm-connectivity-proposals.md` | Product/realtime maintainers | DM delivery policy, server-node/message-node envelope semantics, node-bypassing DM surface retirement, or acceptance criteria change |
| DM encrypted-envelope delivery execution planning authority | `docs/planning/infra-free-dm-connectivity-execution-plan.md` | Delivery/core/realtime maintainers | DM delivery sequencing, task gates, node-bypassing DM surface retirement work, or acceptance evidence changes |
| Dependencies and risk register | `docs/product/04-dependencies-risks.md` | Product/architecture maintainers | Dependency status or risk severity/mitigation changes |
| Iteration execution planning | `docs/planning/iterations/README.md` | Delivery maintainers | Task sequencing, ownership, dependencies, or status changes |
| KPI/SLO benchmark environment profile | `docs/planning/kpi-slo-test-profile.md` | Platform maintainers | Test environment assumptions, load profile, or benchmark matrix changes |
| TURN/NAT constrained-network voice validation profile | `docs/planning/turn-nat-test-profile.md` | Platform/realtime maintainers | NAT scenario matrix, relay expectations, or constrained-network evidence contract changes |
| Local runtime testing profiles, fixtures, multi-instance launch, and network simulation | `docs/planning/local-runtime-testing-plan.md` | Platform/QA maintainers | Local seed profiles, fixture data, runtime profile topology, or network simulation strategy changes |
| Iteration-level change log | `docs/planning/05-iteration-log.md` | Delivery maintainers | Scope, sequencing, status, or decision changes during execution |
| Data lifecycle and retention boundaries | `docs/architecture/02-data-lifecycle-retention-replication.md` | Architecture/API maintainers | Persistence ownership, retention, or reconciliation rules change |
| Rust service migration baseline and file mapping | `docs/architecture/03-rust-service-migration-baseline.md` | Architecture maintainers | Rust service module migration scope, baseline evidence, or mapping changes |
| Communication networking layer architecture and implementation divergence | `docs/architecture/04-communication-networking-layer-plan.md` | Architecture/core/realtime maintainers | Shared communication-layer boundaries, server-node policy graph, discovery/peering/relay/delivery rules, DM/server transport divergence, or networking rollout phases change |
| MVP operational runbook | `docs/operations/01-mvp-runbook.md` | Platform maintainers | Incident/recovery/backup procedures change |
| Dedicated-server deployment baseline | `docs/operations/02-dedicated-server-deployment.md` | Platform maintainers | Dedicated operator bring-up, ingress, administration surface, remote smoke, or deployment-scope assumptions change |
| Release packaging and artifact model | `docs/operations/03-release-packaging.md` | Platform maintainers | Supported release targets, desktop/server artifact boundaries, administration-surface packaging, installer formats, or signing expectations change |
| Private server-node mesh bootstrap | `docs/operations/private-mesh-bootstrap.md` | Platform maintainers | Private mesh node identity, peer invite, static peer, or revocation operations change |
| Local development prerequisites | `docs/operations/dev-prerequisites.md` | Platform maintainers | Required local tooling versions or setup flow changes |
| Migration evidence template | `docs/operations/migration-validation-template.md` | Delivery/platform maintainers | Migration evidence fields, required artifacts, or validator contract changes |
| Readiness corrections and recurrence prevention log | `docs/operations/readiness-corrections-log.md` | Maintainers | Any readiness fix lands or a previously closed finding regresses |
| Recurring quality audit ledgers | `docs/operations/quality-audits/README.md` | Maintainers | Quality audit topic list, status model, rotation policy, or audit ledger protocol changes |
| MVP requirement-to-evidence verification | `docs/testing/01-mvp-verification-matrix.md` | Delivery/QA maintainers | Verification mapping, evidence format, or validator rules change |
| Observability evidence template | `docs/testing/observability-evidence-template.md` | Delivery/QA maintainers | Observability evidence format, provenance fields, or SLO reporting requirements change |
| Crypto profile conformance verification | `docs/testing/crypto-conformance-checklist.md` | QA/core maintainers | Crypto profile requirement, verification steps, or evidence expectations change |
| Contributor workflow and release hygiene | `docs/operations/contributor-guide.md` | Maintainers | Branch/PR policy, validation gates, or release workflow changes |
| Architecture decisions (ADR set) | `docs/architecture/README.md` and `docs/architecture/adr-*.md` | Architecture maintainers | Any accepted/rejected architecture decision |
| Project glossary and canonical terms | `docs/reference/glossary.md` | Product/engineering maintainers | New domain term appears or an existing term meaning changes |
| Documentation topology and ownership | `docs/README.md` | Maintainers | New docs are added, moved, or retired |

## Documentation Structure

- `docs/product/`: product-level references and navigation
- `docs/contracts/`: runtime and target-state contract artifacts plus contract maintenance indexes
- `docs/planning/`: execution planning guidance and templates
- `docs/architecture/`: system overview, architecture specs, and decision records (ADRs)
- `docs/operations/`: contributor and process operations
- `docs/reference/`: shared definitions and reference material
- `docs/testing/`: verification and evidence governance

## Lightweight Governance

- If any `docs/**/*.md`, `docs/**/*.yaml`, `docs/**/*.yml`, or `docs/**/*.json` file other than `docs/README.md` or recurring numbered ledger files under `docs/operations/quality-audits/[0-9][0-9]-*.md` changes, update `last_updated`, `Latest meaningful change`, and any affected links in this file in the same PR.
- If docs are moved or renamed, keep compatibility stubs at old paths for at least one release cycle or two completed iterations, whichever is longer, except when the old path itself preserves retired project-owned API/realtime versioning or a legacy internal compatibility alias.
- Label docs PRs as either `move-only` or `content-change` in the PR body.
- Keep IDs and naming stable (`kebab-case` for docs, `README.md` for folder indexes).

## Canonical Layout

- Product docs live under `docs/product/`.
- Architecture authorities live under `docs/architecture/`.
- Planning boards live under `docs/planning/iterations/`.
- Shared reference docs live under `docs/reference/`.

## Related Documents

- `README.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/product/08-screen-state-spec.md`
- `docs/product/09-configuration-defaults-register.md`
- `docs/contracts/runtime-rest.openapi.yaml`
- `docs/contracts/README.md`
- `docs/contracts/realtime-events-runtime.asyncapi.yaml`
- `docs/contracts/crypto-profile.md`
- `docs/contracts/mvp-rest.openapi.yaml`
- `docs/contracts/realtime-events.asyncapi.yaml`
- `docs/architecture/02-data-lifecycle-retention-replication.md`
- `docs/architecture/01-system-overview.md`
- `docs/architecture/03-rust-service-migration-baseline.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/architecture/adr-0003-rust-service-module-architecture.md`
- `docs/operations/01-mvp-runbook.md`
- `docs/operations/02-dedicated-server-deployment.md`
- `docs/operations/03-release-packaging.md`
- `docs/operations/private-mesh-bootstrap.md`
- `docs/operations/local-runtime-testing-quickstart.md`
- `docs/operations/dev-prerequisites.md`
- `docs/operations/readiness-corrections-log.md`
- `docs/operations/quality-audits/README.md`
- `docs/reference/runtime-config-reference.md`
- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/local-runtime-testing-plan.md`
- `docs/planning/navigation-implementation-plan.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/testing/01-mvp-verification-matrix.md`
- `docs/testing/crypto-conformance-checklist.md`
- `docs/planning/05-iteration-log.md`
- `docs/operations/contributor-guide.md`
