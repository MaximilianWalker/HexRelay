# HexRelay Documentation Index

## Document Metadata

- Doc ID: docs-index
- Owner: HexRelay maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 registered realtime contract and KPI/SLO test profile as canonical planning artifacts.

## Purpose

- Canonical routing for project documentation and update responsibilities.

## Source-of-Truth Matrix

| Topic | Canonical document | Owner | Update trigger |
|---|---|---|---|
| Product intent, constraints, architecture baseline | `docs/product/01-mvp-plan.md` | Product/architecture maintainers | Scope, constraints, architecture, privacy, or security decision changes |
| Product requirements and success metrics | `docs/product/02-prd-v1.md` | Product maintainers | Functional/non-functional requirements, user flows, or success metrics change |
| Iteration 1 API contract baseline | `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml` | API maintainers | Any identity/auth/invite schema or error contract change |
| Realtime event/signaling contract baseline | `docs/contracts/realtime-events-v1.asyncapi.yaml` | Realtime maintainers | Any websocket/signaling event schema change |
| MVP UI navigation and layout authority | `docs/product/07-ui-navigation-spec.md` | Product/design maintainers | Navigation paradigm, screen hierarchy, or hub behavior changes |
| Product clarifications and open questions | `docs/product/03-clarifications.md` | Product maintainers | Any assumption is resolved, added, or materially changed |
| Dependencies and risk register | `docs/product/04-dependencies-risks.md` | Product/architecture maintainers | Dependency status or risk severity/mitigation changes |
| Iteration execution planning | `docs/planning/iterations/README.md` | Delivery maintainers | Task sequencing, ownership, dependencies, or status changes |
| KPI/SLO benchmark environment profile | `docs/planning/kpi-slo-test-profile.md` | Platform maintainers | Test environment assumptions, load profile, or benchmark matrix changes |
| Iteration-level change log | `docs/planning/05-iteration-log.md` | Delivery maintainers | Scope, sequencing, status, or decision changes during execution |
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

## Lightweight Governance

- If canonical docs change, update `last_updated` and affected links in this file in the same PR.
- If docs are moved or renamed, keep compatibility stubs at old paths until at least Iteration 6.
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
- `docs/product/07-ui-navigation-spec.md`
- `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
- `docs/contracts/realtime-events-v1.asyncapi.yaml`
- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/05-iteration-log.md`
- `docs/operations/contributor-guide.md`
