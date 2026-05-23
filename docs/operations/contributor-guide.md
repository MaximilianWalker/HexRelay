# HexRelay Contributor Guide

## Document Metadata

- Doc ID: contributor-guide
- Owner: Maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/operations/contributor-guide.md`

## Quick Context

- Primary edit location for contribution workflow, docs QA checks, and PR hygiene.
- Keep this aligned with `docs/README.md` source-of-truth ownership rules.
- Latest meaningful change: 2026-05-20 clarified repository ownership for top-level `fixtures/`, `tests/`, and executable `scripts/`.

## Purpose

- Define the default contribution workflow for MVP-phase development.
- Keep quality gates deterministic without slowing delivery.

## Repository State

- Current state includes active implementation across web, API, and realtime services.
- Primary product runtime target is bundled desktop local-first operation.
- Dedicated server mode remains a supported path and should be preserved in architecture/API decisions.
- Before planning against current runtime behavior or calling work `ready`, check open `watch` entries in `docs/operations/readiness-corrections-log.md`; current deferred items include broader semantic contract validation beyond current parity checks, process-local realtime websocket abuse-control deployment sensitivity, and docs-governance/process watches.

## Local Development Prerequisites

- Before first setup, verify required local tooling versions in `docs/operations/dev-prerequisites.md`.
- Rust toolchain follows latest stable via `rust-toolchain.toml`; run `rustup toolchain install stable` if local toolchain is missing.

## Branch and PR Workflow

- Use short-lived branches from the default branch.
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
- If any `docs/**/*.md`, `docs/**/*.yaml`, `docs/**/*.yml`, or `docs/**/*.json` file other than `docs/README.md` changes, refresh `docs/README.md` metadata in the same PR because `docs-index-freshness-check` enforces that repo-wide rule.
  - Confirm canonical source-of-truth boundaries are still respected (no duplicate authority across docs).
  - If docs mention smoke/bootstrap flows, state any required temporary config opt-ins explicitly rather than assuming CI-only knowledge.
- For code changes:
  - Run lint, tests, and build for touched projects.
  - Run `npm run security` before opening a PR as the fast local Rust-audit gate.
  - If you want local CI-level security parity, also run the extra security checks listed in `Local CI Parity (Pre-PR)`.
  - Keep security-sensitive data out of logs and fixtures.

## Security Tooling Baseline

- `cargo-audit` is pinned to `0.22.0` via `scripts/security/cargo-audit.mjs` and CI uses the same version.
- `npm run security` covers the Rust dependency audit path used by CI.
- Full CI security parity additionally includes:
  - `node scripts/validators/cargo-audit-ignore.mjs`
  - `npm --prefix apps/web audit --omit=dev --audit-level=high`
  - `semgrep scan --config p/security-audit --error --exclude node_modules --exclude target`
- Temporary cargo-audit ignore exceptions must pass `scripts/validators/cargo-audit-ignore.mjs` expiry checks in CI.
- Current ignore-expiry policy lives only in `scripts/security/advisories.mjs`; do not copy advisory IDs into docs or CI.
- If `npm run setup` fails installing `cargo-audit` because Rust is too old, run `rustup update stable` and retry setup.

## CI Expectations

- GitHub Actions workflow `/.github/workflows/ci.yml` is the canonical MVP gate for Rust and web checks.
- Required jobs include `security-audit`, `rust-check`, `web-check`, `windows-parity-check`, `migration-evidence-check`, `evidence-provenance-check`, `contract-parity-check`, `dm-transport-policy-check`, `docs-index-freshness-check`, `rust-coverage-gate`, and `integration-smoke`.
- `contract-parity-check` now covers route/event/error inventory, exact `CookieAuth`/`BearerAuth` security-scheme parity for routed handlers that use `AuthSession` or the server-membership authorization extractors, public-route auth absence parity for routes that have no session or internal-token auth at runtime, shared REST `ApiError` schema-shape parity, CSRF header absence plus component name/location, conditional-requiredness, and schema-type parity, internal-token request-header absence, requiredness, and schema-type parity, selected auth/CSRF/storage semantics, `401` response presence for session-auth routes and direct unauthorized runtime emitters plus local failure helpers, `500` response presence for session-auth storage paths and non-auth local `internal_error(...)` helper/delegate flows, `400` response presence for local parse/normalize/validation helper/delegate flows including GET cursor and limit parsing, query-parameter semantic parity for requiredness, schema type, enum domains, selected string patterns, blank-search normalization, case-insensitive matching, trim-before-enum normalization, and reject-backed numeric bounds on the main filter/pagination surfaces, extractor-backed `403`/`404` error-response presence for server-membership authorization routes, OpenAPI path parameter presence, requiredness, schema-type, and selected format parity for routed handlers that directly use `Path<...>` extractors, OpenAPI query parameter presence for routed handlers that directly use `Query<...>` extractors, OpenAPI `requestBody` presence and requiredness for routed API handlers that accept `Json<...>` request extractors including inline and component-referenced bodies, OpenAPI JSON request media-type exclusivity for routed `Json<...>` extractors, OpenAPI `requestBody` absence for routed handlers without request-body extractors, high-signal success-response presence for routed handlers with confidently inferred `2xx` outcomes, response-builder success-schema parity for local `Json(body).into_response()` handlers, tracked REST response-header schema-type parity, selected routed error-response presence for directly emitted `400`/`403`/`404`/`409`/`429` paths, route-level error-example parity for the tracked high-signal auth/social/DM/server routes, selected tracked REST DTO date-time format, scalar-bound, enum-domain including DM privacy-policy policy modes, string-pattern, selected array item-pattern, and `serde(default)` request-field optionality parity, selected realtime semantic parity for the receive-side `realtime.connected`, `presence.updated`, and server-channel message event envelopes, current send-side signaling guard semantics for `call.signal.offer`, `call.signal.answer`, and `call.signal.ice_candidate` (authenticated `from_identity_id` parity plus self-targeted-only delivery support), current send-side signaling success-envelope parity for those same signaling events, and shared realtime `error` envelope/data parity. Success-body closeout work should keep documenting branch-specific `200`/`201` payload meaning where one schema serves multiple runtime outcomes, lifecycle states, idempotent success paths, setup-result branches, intentionally indistinguishable auth flows, or sorted/empty-list read semantics.
- `contract-parity-check` also enforces required-field, nullable-field, field-type, selected date-time format, selected scalar-bound, selected enum-domain, selected string-pattern, selected array item-pattern, nested array item-schema, and referenced item-field parity for a tracked set of high-signal REST DTO schemas: `AuthVerifyRequest`, `AuthVerifyResponse`, `SessionValidateResponse`, `InviteCreateRequest`, `InviteCreateResponse`, `ServerChannelMessageCreateRequest`, `ServerChannelMessageEditRequest`, `FriendRequestCreateRequest`, `DmPolicy`, `DmPolicyUpdate`, `DmFanoutDispatchRequest`, `DmFanoutDispatchResponse`, `DmFanoutCatchUpRequest`, `DmFanoutCatchUpItem`, `DmFanoutCatchUpResponse`, `DmThreadMarkReadRequest`, and `DmThreadMarkReadResponse`.
- Current enforced backend coverage threshold is 80% and must remain paired with meaningful test additions when enforcement changes.
- Current enforced web coverage thresholds are 65% for lines/statements/functions and 60% for branches in `apps/web/vitest.config.ts`, and threshold increases must ship with the test additions that justify them.
- Rust gate runs `fmt`, `clippy`, and `test` for `services/api-rs` and `services/realtime-rs`.
- Web gate runs `lint`, `test:coverage`, and `build` for `apps/web`.
- Windows parity gate runs `npm run setup`, validates runtime/network profile definitions, and runs `npm run test -- --skip-service-backed-tests` on `windows-latest`; Linux CI remains responsible for DB/Redis-backed Rust tests and integration smoke.
- Integration smoke always uploads CI evidence artifacts at `evidence/ci/<run_id>/`.
- Missing required lockfiles or missing `lint`/`test:coverage`/`build` scripts fail CI with actionable errors.

Non-localizable CI checks:
- `migration-evidence-check` requires PR base/head SHAs from CI context.
- `integration-smoke` artifact upload path is CI-owned (`evidence/ci/<run_id>/`).

## Local CI Parity (Pre-PR)

Required local checks before opening a PR:

- `npm run check -- --skip-service-backed-tests`
- `cargo llvm-cov --workspace --all-features --fail-under-lines 80` when coverage-relevant Rust code changes
- `npm --prefix apps/web audit --omit=dev --audit-level=high` when web dependencies change
- `semgrep scan --config p/security-audit --error --exclude node_modules --exclude target` when Semgrep is installed locally

CI-owned checks (informational for local parity):
- CI artifact upload and retention under `evidence/ci/<run_id>/`
- PR-context dependent SHA resolution in workflow jobs

Run from repository root. `npm run check` resolves the base SHA automatically; set `BASE_SHA` and `HEAD_SHA` only when you need to compare a specific range.

```text
npm run check -- --skip-service-backed-tests
```

- `npm run check` is the canonical cross-platform local gate for repo-owned validators, profile validation, contract fixture regressions, Rust checks, and web lint/test/build.
- If your change affects auth/realtime startup behavior, run `npm --prefix apps/web run e2e:smoke` after API and realtime are healthy.

## Local Happy Path and Triage

1. `npm run setup`
2. `npm run start`
3. Verify `curl -fsS "http://127.0.0.1:8080/health"` and `curl -fsS "http://127.0.0.1:8081/health"`
4. `npm --prefix apps/web run e2e:smoke`
5. If startup or smoke fails, follow `docs/operations/01-mvp-runbook.md` recovery and rollback sections.

## Docs QA Checklist

- Metadata block is present and complete (`Doc ID`, `Owner`, `Status`, `Scope`, `last_updated`, `Source of truth`).
- Canonical ownership is explicit in `docs/README.md` source-of-truth matrix.
- New links point to canonical indexes where possible (for example, iteration index over repeated board lists).
- Related documents section is updated when new canonical docs are introduced.
- Runtime/deployment wording matches `docs/architecture/adr-0002-runtime-deployment-modes.md` and does not introduce conflicting authority text.
- Recurring readiness findings are recorded and closed in `docs/operations/readiness-corrections-log.md`.

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
