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

## 4) Readiness Feedback Loop (Required)

- When a readiness finding is fixed, record it in `docs/operations/readiness-corrections-log.md` in the same change.
- For repeated findings, add or tighten a durable rule in `AGENTS.md` or the canonical owning document in the same change.
- Before opening a new readiness audit cycle, check `docs/operations/readiness-corrections-log.md` and treat open findings as first-pass candidates.
- Do not re-open previously closed findings unless new code/docs changes invalidate the prior fix; if invalidated, record the regression explicitly in the log.
