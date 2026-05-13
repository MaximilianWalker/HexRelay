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
| QA-05-20260513-missing-ui-render-coverage | P2 | found | Core web UI flows lack deterministic render/browser regression tests. | `apps/web/package.json:14-15` runs Vitest coverage and maps `e2e:smoke` to a Node script; `apps/web/scripts/e2e-smoke.mjs:19,47,80,94,106,130` uses `fetch` and `ws` against API/realtime without rendering the app; `rg --files apps/web -g '*.test.tsx' -g '!node_modules'` returned no component/page tests while `apps/web/app` and `apps/web/components` contain 14 TSX page/component files; `docs/testing/01-mvp-verification-matrix.md:30` leaves Navigation and hubs to a manual screenshot checklist. | Add browser or component-level regression tests for workspace navigation, onboarding, contacts, and DM screen states, then wire them into the web CI gate. | 2026-05-13T00:31:36Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T00:31:36Z | Codex automation | Added 2 P2 found findings about Linux-only CI test gates and missing deterministic web UI render coverage. |
