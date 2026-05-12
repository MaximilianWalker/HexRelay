# Quality Audit Ledgers

## Document Metadata

- Doc ID: quality-audit-ledgers
- Owner: Maintainers
- Status: active
- Scope: repository
- last_updated: 2026-05-12
- Source of truth: `docs/operations/quality-audits/README.md`

## Quick Context

- These files are recurring audit ledgers for the 25 repository quality topics.
- Each automation run audits exactly one topic, selected by least-recently-audited order.
- Existing active findings in the selected topic file must be loaded first and must not be rediscovered as new findings.
- The automation records findings only; it must not fix source code, push branches, or open PRs unless explicitly requested later.

## Run Protocol

1. Read this index and all topic metadata.
2. Select the topic whose topic file has the oldest `last_audited` value. Treat `never` as oldest. Break ties by the lowest numeric topic id.
3. Read the selected topic file before inspecting the repo.
4. Treat `found`, `confirmed`, and `watch` rows as known issues. Update them only when new evidence changes priority, status, or next step.
5. Investigate repo-wide code, tests, docs, scripts, CI, and contracts through the selected topic only.
6. Edit only the selected topic ledger unless the audit protocol itself is intentionally changed.
7. Commit only when the selected topic ledger content changes.

## Status Model

| Status | Meaning |
|---|---|
| `found` | Newly identified, not yet independently confirmed |
| `confirmed` | Valid issue with actionable repository evidence |
| `watch` | Real concern, but deferred or unsafe to fix immediately |
| `fixed` | No longer present |
| `invalid` | Rechecked and not a real issue |
| `superseded` | Replaced by a clearer or newer finding |

## Priority Model

| Priority | Meaning |
|---|---|
| `P0` | Severe security, data loss, privacy, or correctness risk |
| `P1` | High-impact reliability, architecture, or core workflow risk |
| `P2` | Maintainability, test, DX, observability, or medium product risk |
| `P3` | Low-risk polish, docs clarity, cleanup, or future hardening |

## Topic Ledgers

| ID | Topic | Ledger |
|---|---|---|
| 01 | Correctness | `docs/operations/quality-audits/01-correctness.md` |
| 02 | Code Readability | `docs/operations/quality-audits/02-code-readability.md` |
| 03 | Architecture | `docs/operations/quality-audits/03-architecture.md` |
| 04 | Maintainability | `docs/operations/quality-audits/04-maintainability.md` |
| 05 | Test Quality | `docs/operations/quality-audits/05-test-quality.md` |
| 06 | Error Handling | `docs/operations/quality-audits/06-error-handling.md` |
| 07 | Security | `docs/operations/quality-audits/07-security.md` |
| 08 | Performance | `docs/operations/quality-audits/08-performance.md` |
| 09 | Reliability | `docs/operations/quality-audits/09-reliability.md` |
| 10 | Observability | `docs/operations/quality-audits/10-observability.md` |
| 11 | API Design | `docs/operations/quality-audits/11-api-design.md` |
| 12 | Data Design | `docs/operations/quality-audits/12-data-design.md` |
| 13 | Dependency Hygiene | `docs/operations/quality-audits/13-dependency-hygiene.md` |
| 14 | Developer Experience | `docs/operations/quality-audits/14-developer-experience.md` |
| 15 | Documentation | `docs/operations/quality-audits/15-documentation.md` |
| 16 | Build and Release Quality | `docs/operations/quality-audits/16-build-and-release-quality.md` |
| 17 | UX / Product Quality | `docs/operations/quality-audits/17-ux-product-quality.md` |
| 18 | Accessibility | `docs/operations/quality-audits/18-accessibility.md` |
| 19 | Portability | `docs/operations/quality-audits/19-portability.md` |
| 20 | Scalability | `docs/operations/quality-audits/20-scalability.md` |
| 21 | Privacy | `docs/operations/quality-audits/21-privacy.md` |
| 22 | Concurrency and State | `docs/operations/quality-audits/22-concurrency-and-state.md` |
| 23 | Code Style Consistency | `docs/operations/quality-audits/23-code-style-consistency.md` |
| 24 | Reviewability | `docs/operations/quality-audits/24-reviewability.md` |
| 25 | Long-Term Evolvability | `docs/operations/quality-audits/25-long-term-evolvability.md` |

## Commit Policy

- Use branch `codex/quality-audits`.
- Commit only changed audit ledger files.
- Use commit message format: `docs: update <topic> quality audit ledger`.
- Do not push, open PRs, or modify runtime/product code during automated audit runs.
