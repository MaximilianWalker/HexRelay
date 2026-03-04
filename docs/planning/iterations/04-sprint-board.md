# HexRelay Iteration 4 Sprint Board

## Document Metadata

- Doc ID: iteration-04-sprint-board
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/planning/iterations/04-sprint-board.md`
- Board status: planned

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 added explicit iteration entry criteria and exit evidence requirements.

## Iteration Scope

Scope: Iteration 4 (Weeks 10-12) from `docs/product/01-mvp-plan.md`.

## Goals

- Deliver federation-lite node discovery and portability mechanisms.
- Ship profile capsule replication and full migration workflows.
- Complete reliability/observability and beta hardening gates.

## Status Legend

- `planned`: not started
- `in_progress`: active implementation
- `done`: acceptance criteria met
- `blocked`: cannot proceed due to unresolved dependency

## Effort Scale

- S: 0.5-1 day
- M: 1-2 days
- L: 2-4 days
- XL: 4-6 days

## Backlog

| ID | Task | Epic/Story | Effort | Owner | Dependencies | Acceptance Criteria |
|---|---|---|---|---|---|---|
| T7.1.1 | Build JSON export package for account/server data and media index | E7 / S7.1 | L | API | T6.1.1, T6.3.1 | Export artifacts are deterministic and schema-versioned |
| T7.1.2 | Build import flow with id remapping and conflict handling | E7 / S7.1 | XL | API | T7.1.1 | Import succeeds with conflict policy and dry-run report |
| T7.2.1 | Implement signed registry parser and periodic fetch | E7 / S7.2 | M | API | T2.1.1, T2.3.1 | Registry entries validate signatures and TTLs |
| T7.2.2 | Add node discovery UI and server join flow | E7 / S7.2 | L | Web | T7.2.1 | Users can browse/join nodes with fingerprint verification |
| T7.3.1 | Implement node metadata publishing CLI/docs | E7 / S7.3 | M | Platform | T7.2.1 | Published metadata passes registry schema/signature validation in CLI integration tests |
| T7.4.1 | Implement signed public profile sync + version conflict handling | E7 / S7.4 | L | API | T2.1.1, T2.3.1 | Latest valid signed public profile converges across nodes |
| T7.4.2 | Implement encrypted private profile replica sync + restore flow | E7 / S7.4 | L | API | T7.4.1 | Private capsule replicas restore correctly on a new device and fail closed on signature/version mismatch |
| T7.5.1 | Define encrypted migration bundle format (`.hxb`) and verification rules | E7 / S7.5 | M | Core | T7.4.2 | Bundle spec includes encryption, signatures, versioning, integrity checks |
| T7.5.2 | Implement full migration export (identity/profile/settings/local state/optional media cache) | E7 / S7.5 | XL | Core | T7.5.1 | Export completes with progress and cryptographic integrity report |
| T7.5.3 | Implement migration import + reconciliation with server state | E7 / S7.5 | XL | API | T7.5.2, T7.1.2 | Import restores state and reconciles missing data from nodes with deterministic conflict logs |
| T7.5.4 | Implement LAN direct transfer + encrypted file fallback | E7 / S7.5 | L | Core | T7.5.2 | Users can migrate over LAN or manual encrypted file path |
| T7.5.5 | Implement optional cutover revoke for old device sessions | E7 / S7.5 | M | API | T7.5.3 | User can revoke previous device sessions after successful migration |
| T8.1.1 | Add OTel traces/metrics and simple dashboard views | E8 / S8.1 | M | Platform | T5.3.1, T6.3.1 | Dashboards expose p95 latency, error rate, auth success, E2EE DM success |
| T8.2.1 | Define/enforce SLO alerts for latency and error budgets | E8 / S8.2 | M | Platform | T8.1.1 | Alert rules trigger correctly in staging fault-injection tests with <= 2 minute detection delay for injected breaches |
| T8.3.1 | Publish beta admin guide + user onboarding docs | E8 / S8.3 | M | Docs | T7.5.3, T8.2.1 | Beta docs cover setup, identity recovery, migration, and troubleshooting |

## Task Touchpoints and Validation Gates

| Task | Target touchpoints | Validation |
|---|---|---|
| T7.1.1-T7.1.2 | Export/import schemas, remapping engine, dry-run reporting | Deterministic export hash and import conflict-policy tests pass |
| T7.2.1-T7.2.2 | Registry parser/fetcher and discovery UI join flow | Signature/TTL verification suite passes; join flow enforces fingerprint verification |
| T7.3.1 | Metadata publishing CLI and schema/signature validators | CLI integration tests confirm valid publish and invalid metadata rejection |
| T7.4.1-T7.4.2 | Profile sync pipelines and private replica restore flow | Cross-node convergence and fail-closed restore tests pass |
| T7.5.1-T7.5.5 | Migration bundle spec/export/import/reconciliation/cutover | Three migration scenarios pass (LAN + encrypted file + cutover revoke) |
| T8.1.1-T8.2.1 | OTel instrumentation, dashboards, alert rules | Staging fault-injection confirms metrics and <= 2 minute alert detection |
| T8.3.1 | Beta admin/user docs package | Docs review checklist passes for setup, recovery, migration, troubleshooting coverage |

## Entry Criteria

- Iteration 3 voice/media/local-controls are complete and regression-tested.
- Migration conflict resolution policy is defined before `T7.1.2` and `T7.5.3` finalization: user-signed profile state is canonical; server-owned security/membership fields remain server-authoritative (resolved by `C-014`).
- SLO alert test profile is fixed by `docs/planning/kpi-slo-test-profile.md` before `T8.2.1` execution (resolved by `C-013`).

## Exit Evidence

- Evidence pack includes migration scenario runs (LAN/file/cutover) with deterministic conflict logs.
- Discovery ingestion and publish validation outputs are linked for operator flows.
- Observability dashboards and alert fault-injection reports are attached to release readiness notes.

## UI and Flow State Mapping

| Flow | Required states (authoritative set in `docs/product/08-screen-state-spec.md`) |
|---|---|
| Discovery and server join | loading, search_no_results, permission_denied, error |
| Migration export/import | export_running, import_running, conflict_review, reconcile_running, completed, failed |
| Observability/SLO review | loading, degraded, breached, recovered |

## Evidence Ledger

| Task set | Evidence artifact path | Validator |
|---|---|---|
| T7.1.x-T7.3.1 | `evidence/iteration-04/discovery-portability/` | registry and migration contract suites |
| T7.4.x-T7.5.x | `evidence/iteration-04/profile-migration/` | replication + reconcile scenario matrix |
| T8.1.x-T8.3.1 | `evidence/iteration-04/observability-beta/` | fault-injection + docs checklist |

## In Progress

| ID | Task | Status | Notes |
|---|---|---|---|
| None | - | Not started | Awaiting Iteration 3 completion |

## Done

| ID | Task | Completed In | Notes |
|---|---|---|---|
| None | - | - | - |

## Suggested Sprint Sequencing

Week 10:

- T7.1.1 -> T7.1.2
- T7.2.1 -> T7.2.2
- T7.3.1
- Start T7.4.1

Week 11:

- T7.4.2
- T7.5.1 -> T7.5.2
- T8.1.1

Week 12:

- T7.5.3 -> T7.5.4 -> T7.5.5
- T8.2.1
- T8.3.1
- Beta readiness review and iteration demo

## Iteration 4 Exit Checklist

- Discovery list ingestion and secure join flow are working.
- Profile capsule replication and restore pass cross-node tests.
- Full migration works in both LAN and encrypted file modes.
- Cutover revoke successfully invalidates old sessions.
- Observability and SLO alerting satisfy MVP exit metrics.

## Execution Notes

- Never include raw invite secrets, identity keys, or migration passphrases in logs.
- Keep bundle format backward-compatible with explicit schema versions.
- For migration UX, surface progress phases clearly (export, transfer, import, reconcile).
- Core owns bundle/export and API owns import/reconciliation (`T7.5.2`, `T7.5.3`) with explicit interface contracts.
- Tag PRs and commits with task IDs (`T7.x.x`, `T8.x.x`).

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/08-screen-state-spec.md`
- `docs/architecture/02-data-lifecycle-retention-replication.md`
- `docs/testing/01-mvp-verification-matrix.md`
- `docs/reference/glossary.md`
