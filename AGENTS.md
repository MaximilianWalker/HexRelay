This directory follows the global AGENTS.md at `~/.config/opencode/AGENTS.md`.
Only project-specific constraints are defined here.

# HexRelay Repo Rules

## 1) Scope

- Primary goal: build an open-source, Discord-like communication platform with strong user data ownership.
- MVP focus: reliable friends, DMs, guild channels, and voice before federation complexity.
- Architecture baseline: Rust-first backend services with desktop local-first runtime packaging and reusable web UI layer.
- Development baseline: monorepo scaffolds, local infra compose stack, and CI gates are active and should be maintained.

## 2) Editing Boundaries

- Keep architecture docs current in `docs/product/01-mvp-plan.md` when major decisions change.
- Keep requirements and dependency/risk state current in `docs/product/02-prd-v1.md` and `docs/product/04-dependencies-risks.md` when behavior changes.
- Prefer minimal diffs and avoid broad refactors during MVP setup.

## 3) Product Guardrails

- Do not introduce paywalled core communication features.
- Preserve portability and export/import capabilities in all storage decisions.
- Treat decentralization as phased delivery to avoid blocking UX quality.
- Keep DM transport direct user-to-user; do not reintroduce guild/server relay for DM payloads.
- Reject networking solutions that require always-on third-party or project-operated infrastructure for DM connectivity (for example STUN/TURN relay services).
- Prefer infrastructure-free peer connectivity modes only; if direct connection cannot be established, fail explicitly with user guidance rather than introducing infra fallback.

## 4) Readiness Feedback Loop (Required)

- When a readiness finding is fixed, record it in `docs/operations/readiness-corrections-log.md` in the same change.
- For repeated findings, add or tighten a durable rule in `AGENTS.md` or the canonical owning document in the same change.
- Before opening a new readiness audit cycle, check `docs/operations/readiness-corrections-log.md` and treat open findings as first-pass candidates.
- Do not re-open previously closed findings unless new code/docs changes invalidate the prior fix; if invalidated, record the regression explicitly in the log.

## 5) Standard Readiness Execution Flow (Repeatable)

- When asked to check readiness before continuing development, run two parallel subagent audits:
  - docs audit (`documentation-governor`)
  - API/realtime code audit (`code-reviewer`)
- After audits, always run a strict revalidation pass:
  1. create a todo list with each reported issue,
  2. classify each issue as `valid` vs `nitpick` with repository evidence,
  3. fix only valid, high-signal items,
  4. append each correction/reason to `docs/operations/readiness-corrections-log.md`.
- If an issue is real but not safe for a minimal pass (architectural/high-risk), do not force a partial fix; log it as `watch` in the readiness corrections log with explicit deferral reason.
- For docs-only readiness passes, skip unnecessary code test reruns; for code-touching passes, run formatter/tests/clippy for touched services before commit.

## 6) Protected Branch Delivery Flow (Repeatable)

- Default sequence for delivering changes:
  1. commit on current branch,
  2. attempt push,
  3. if `master` is blocked by protection, create `chore/*` branch and push,
  4. open PR,
  5. watch required checks to completion,
  6. resolve unresolved review threads,
  7. merge PR,
  8. sync local `master`,
  9. delete local and remote feature branch.
- Do not skip the check-watch step; merge only after required checks are green and conversation-resolution requirements are satisfied.
