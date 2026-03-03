# HexRelay Iteration 4 Sprint Board

Scope: Iteration 4 (Weeks 10-12) from `MVP_PLAN.md`.

Goals:

- Deliver federation-lite node discovery and portability mechanisms.
- Ship profile capsule replication and full migration workflows.
- Complete reliability/observability and beta hardening gates.

Effort scale:

- S: 0.5-1 day
- M: 1-2 days
- L: 2-4 days
- XL: 4-6 days

## Backlog

| ID | Task | Epic/Story | Effort | Owner | Dependencies | Acceptance Criteria |
|---|---|---|---|---|---|---|
| T7.1.1 | Build JSON export package for account/server data and media index | E7 / S7.1 | L | API | Iteration 3 complete | Export artifacts are deterministic and schema-versioned |
| T7.1.2 | Build import flow with id remapping and conflict handling | E7 / S7.1 | XL | API | T7.1.1 | Import succeeds with conflict policy and dry-run report |
| T7.2.1 | Implement signed registry parser and periodic fetch | E7 / S7.2 | M | API | Iteration 1 identity primitives | Registry entries validate signatures and TTLs |
| T7.2.2 | Add node discovery UI and server join flow | E7 / S7.2 | L | Web | T7.2.1 | Users can browse/join nodes with fingerprint verification |
| T7.3.1 | Implement node metadata publishing CLI/docs | E7 / S7.3 | M | Platform | T7.2.1 | Admins can publish compliant discovery metadata |
| T7.4.1 | Implement signed public profile sync + version conflict handling | E7 / S7.4 | L | API | Iteration 1 identity | Latest valid signed public profile converges across nodes |
| T7.4.2 | Implement encrypted private profile replica sync + restore flow | E7 / S7.4 | L | API/Web | T7.4.1 | Private capsule replicas restore correctly on a new device |
| T7.5.1 | Define encrypted migration bundle format (`.hxb`) and verification rules | E7 / S7.5 | M | Core | T7.4.2 | Bundle spec includes encryption, signatures, versioning, integrity checks |
| T7.5.2 | Implement full migration export (identity/profile/settings/local state/optional media cache) | E7 / S7.5 | XL | Web/Core | T7.5.1 | Export completes with progress and cryptographic integrity report |
| T7.5.3 | Implement migration import + reconciliation with server state | E7 / S7.5 | XL | Web/API | T7.5.2, T7.1.2 | Import restores state and reconciles missing data from nodes |
| T7.5.4 | Implement LAN direct transfer + encrypted file fallback | E7 / S7.5 | L | Core | T7.5.2 | Users can migrate over LAN or manual encrypted file path |
| T7.5.5 | Implement optional cutover revoke for old device sessions | E7 / S7.5 | M | API | T7.5.3 | User can revoke previous device sessions after successful migration |
| T8.1.1 | Add OTel traces/metrics and simple dashboard views | E8 / S8.1 | M | Platform | Core services stable | Dashboards expose p95 latency, error rate, auth success, E2EE DM success |
| T8.2.1 | Define/enforce SLO alerts for latency and error budgets | E8 / S8.2 | M | Platform | T8.1.1 | Alert rules trigger correctly in staging fault-injection tests |
| T8.3.1 | Publish beta admin guide + user onboarding docs | E8 / S8.3 | M | Docs | T7.5.3, T8.2.1 | Beta docs cover setup, identity recovery, migration, and troubleshooting |

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
- Tag PRs and commits with task IDs (`T7.x.x`, `T8.x.x`).
