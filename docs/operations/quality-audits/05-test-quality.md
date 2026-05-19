# Test Quality Audit

## Metadata

- topic_id: 05-test-quality
- topic: Test Quality
- last_audited: 2026-05-13T00:31:36Z
- source_of_truth: `docs/operations/quality-audits/05-test-quality.md`

## Investigation Focus

- Review whether critical behavior has deterministic unit, integration, contract, or end-to-end coverage.
- Flag brittle tests, weak assertions, missing regressions for fixed bugs, and coverage gates that do not protect important behavior.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-05-20260513-linux-only-ci-tests | P2 | found | CI test gates do not exercise the mandatory Windows path. | `AGENTS.md:12` requires Windows and Linux as first-class testing targets; `package.json:35-36` exposes separate `test:windows` and `test:unix` wrappers; `.github/workflows/ci.yml:23,61,125,145,165,186,206,218,239,308,362` all use `ubuntu-latest`, and `rg -n "windows-latest" .github/workflows package.json scripts docs` returned no CI runner coverage. | Add a Windows CI job or matrix lane that runs the PowerShell-backed test path, with any heavier Linux-only smoke documented separately. | 2026-05-13T00:31:36Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-05-20260513-missing-ui-render-coverage | P2 | fixed | Core web UI flows lacked deterministic render/browser regression tests. | Added `apps/web/lib/ui-render.test.ts` covering workspace navigation, identity onboarding, contacts, and routed private-message session-required render states; `apps/web/package.json` already wires Vitest coverage into the CI `web-check`, and `docs/testing/01-mvp-verification-matrix.md` now records the deterministic render validator alongside manual screenshots. Pre-fix focused check `rg --files apps/web -g '*.test.tsx' -g '!node_modules'` reproduced the selector's missing-render-test evidence, and the new focused render test passes under `npm --prefix apps/web run test -- lib/ui-render.test.ts`. | 2026-05-19 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T00:31:36Z | Codex automation | Added 2 P2 found findings about Linux-only CI test gates and missing deterministic web UI render coverage. |
