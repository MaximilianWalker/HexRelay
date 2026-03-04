# HexRelay Contributor Guide

## Document Metadata

- Doc ID: contributor-guide
- Owner: Maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/operations/contributor-guide.md`

## Quick Context

- Primary edit location for contribution workflow, docs QA checks, and PR hygiene.
- Keep this aligned with `docs/README.md` source-of-truth ownership rules.
- Latest meaningful change: 2026-03-04 documentation standardization pass.

## Purpose

- Define the default contribution workflow for MVP-phase development.
- Keep quality gates deterministic without slowing delivery.

## Repository State

- Current state is documentation-first.
- Implementation scaffold is planned in Iteration 1 and not fully committed yet.
- Until scaffold lands, documentation changes are valid contributions.

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
- For code changes (once scaffold exists):
  - Run lint, tests, and build for touched projects.
  - Keep security-sensitive data out of logs and fixtures.

## Docs QA Checklist

- Metadata block is present and complete (`Doc ID`, `Owner`, `Status`, `Scope`, `last_updated`, `Source of truth`).
- Canonical ownership is explicit in `docs/README.md` source-of-truth matrix.
- New links point to canonical indexes where possible (for example, iteration index over repeated board lists).
- Related documents section is updated when new canonical docs are introduced.

## PR Checklist

- Problem and intent are clear.
- Scope is minimal and matches the task.
- Related docs are updated in the same PR.
- Any architecture-impacting change includes an ADR in `docs/architecture/`.
- New terms are added to `docs/reference/glossary.md` when needed.

## Release Hygiene (MVP)

- Merge only when required checks pass.
- Prefer merge cadence tied to iteration milestones.
- For risky changes, include rollback notes in PR description.

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/product/01-mvp-plan.md`
- `docs/planning/05-iteration-log.md`
