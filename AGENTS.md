This directory follows the global AGENTS.md at `~/.config/opencode/AGENTS.md`.
Only project-specific constraints are defined here.

# HexRelay Repo Rules

## 1) Scope

- Primary goal: build an open-source, Discord-like communication platform with strong user data ownership.
- MVP focus: reliable friends, DMs, guild channels, and voice before federation complexity.
- Architecture baseline: Rust-first backend services with a web-first client.

## 2) Editing Boundaries

- Keep architecture docs current in `MVP_PLAN.md` when major decisions change.
- Prefer minimal diffs and avoid broad refactors during MVP setup.

## 3) Product Guardrails

- Do not introduce paywalled core communication features.
- Preserve portability and export/import capabilities in all storage decisions.
- Treat decentralization as phased delivery to avoid blocking UX quality.
