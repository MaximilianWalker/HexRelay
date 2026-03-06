# HexRelay Contributor Guide

## Document Metadata

- Doc ID: contributor-guide
- Owner: Maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-06
- Source of truth: `docs/operations/contributor-guide.md`

## Quick Context

- Primary edit location for contribution workflow, docs QA checks, and PR hygiene.
- Keep this aligned with `docs/README.md` source-of-truth ownership rules.
- Latest meaningful change: 2026-03-06 added explicit local development prerequisite baseline and pinned Rust toolchain guidance.

## Purpose

- Define the default contribution workflow for MVP-phase development.
- Keep quality gates deterministic without slowing delivery.

## Repository State

- Current state includes active implementation across web, API, and realtime services.
- Primary product runtime target is bundled desktop local-first operation.
- Dedicated server mode remains a supported path and should be preserved in architecture/API decisions.

## Local Development Prerequisites

- Before first setup, verify required local tooling versions in `docs/operations/dev-prerequisites.md`.
- Rust toolchain is pinned via `rust-toolchain.toml`; use `rustup update stable` if local toolchain is out of date.

## Branch and PR Workflow

- Use short-lived branches from `main`.
- Suggested branch naming: `feat/<scope>`, `fix/<scope>`, `docs/<scope>`, `chore/<scope>`.
- Keep each PR scoped to one main task or one coherent doc update.
- Reference the task ID as defined in the active sprint board in PR title/body when applicable.

## Commit Policy

- Keep commits focused and reviewable.
- Include DCO sign-off on each commit (`Signed-off-by:` trailer).
- Follow the repository license/contribution baseline: AGPL-3.0 and DCO, no CLA for MVP.

## Validation Expectations

- For docs-only changes:
  - Verify links and paths resolve.
  - Keep metadata and `last_updated` fields accurate.
  - Confirm canonical source-of-truth boundaries are still respected (no duplicate authority across docs).
- For code changes:
  - Run lint, tests, and build for touched projects.
  - Run `npm run security` before opening a PR.
  - Keep security-sensitive data out of logs and fixtures.

## Security Tooling Baseline

- `cargo-audit` is pinned to `0.22.0` via `scripts/ensure-cargo-audit.sh` and CI uses the same version.
- If `npm run setup` fails installing `cargo-audit` because Rust is too old, run `rustup update stable` and retry setup.

## CI Expectations

- GitHub Actions workflow `/.github/workflows/ci.yml` is the canonical MVP gate for Rust and web checks.
- Rust gate runs `fmt`, `clippy`, and `test` for `services/api-rs` and `services/realtime-rs`.
- Web gate runs `lint`, `test:coverage`, and `build` for `apps/web`.
- Missing required lockfiles or missing `lint`/`test`/`build` scripts fail CI with actionable errors.

## Docs QA Checklist

- Metadata block is present and complete (`Doc ID`, `Owner`, `Status`, `Scope`, `last_updated`, `Source of truth`).
- Canonical ownership is explicit in `docs/README.md` source-of-truth matrix.
- New links point to canonical indexes where possible (for example, iteration index over repeated board lists).
- Related documents section is updated when new canonical docs are introduced.
- Runtime/deployment wording matches `docs/architecture/adr-0002-runtime-deployment-modes.md` and does not introduce conflicting authority text.

## PR Checklist

- Problem and intent are clear.
- Scope is minimal and matches the task.
- Related docs are updated in the same PR.
- Any architecture-impacting change includes an ADR in `docs/architecture/`.
- New terms are added to `docs/reference/glossary.md` when needed.
- Any `services/api-rs/migrations/*.sql` change includes an updated evidence artifact at `evidence/migrations/<migration>.md`.

## Release Hygiene (MVP)

- Merge only when required checks pass.
- Prefer merge cadence tied to iteration milestones.
- For risky changes, include rollback notes in PR description.

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/product/01-mvp-plan.md`
- `docs/planning/05-iteration-log.md`
