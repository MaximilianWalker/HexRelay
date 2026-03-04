# HexRelay Iteration 1 Sprint Board

## Document Metadata

- Doc ID: iteration-01-sprint-board
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/planning/iterations/01-sprint-board.md`
- Board status: planned

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 added explicit iteration entry criteria and exit evidence requirements.

## Iteration Scope

Scope: Iteration 1 (Weeks 1-3) from `docs/product/01-mvp-plan.md`.

## Goals

- Stand up a working local development stack.
- Ship portable identity, invite auth, and session security.
- Lock CI quality gates and environment config discipline.

## Status Legend

- `planned`: not started
- `in_progress`: active implementation
- `done`: acceptance criteria met
- `blocked`: cannot proceed due to unresolved dependency

## Effort Scale

- S: 0.5-1 day
- M: 1-2 days
- L: 2-4 days
- XL: 4-6 days

## Backlog

| ID | Task | Epic/Story | Effort | Owner | Dependencies | Acceptance Criteria |
|---|---|---|---|---|---|---|
| T1.1.1 | Create monorepo layout (`apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`) | E1 / S1.1 | M | Core | None | Workspace boots with install commands and project conventions documented |
| T1.1.2 | Add Docker Compose for Postgres, Redis, object storage emulator, coturn | E1 / S1.1 | L | Core | T1.1.1 | `docker compose up` starts all infra and health checks pass |
| T1.1.3 | Add setup/run/test scripts | E1 / S1.1 | M | Core | T1.1.1, T1.1.2 | One-command local startup works from clean checkout |
| T1.2.1 | Configure CI matrix (Rust + web lint/test/build) | E1 / S1.2 | L | Platform | T1.1.1 | PR checks fail on lint/test/build errors and block merges |
| T1.2.2 | Publish Iteration 1 OpenAPI contract artifact | E1 / S1.2 | S | API | T1.1.1 | `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml` is committed and referenced by T2 tasks |
| T1.3.1 | Add env schema validation and config templates | E1 / S1.3 | M | Platform | T1.1.1 | Invalid env values fail fast at startup with actionable errors |
| T2.1.1 | Implement key identity schema + key registration endpoints | E2 / S2.1 | XL | API | T1.1.1, T1.1.2, T1.2.2 | Identity keys can be registered and validated with tests; schema follows MVP Crypto Profile v1 in `docs/product/01-mvp-plan.md` |
| T2.1.2 | Build client key generation/import + secure local key storage | E2 / S2.1 | L | Web | T2.1.1 | User can create or import identity and keep key material encrypted locally |
| T2.1.3 | Implement multi-persona profile switching and session isolation | E2 / S2.1 | M | Web | T2.1.2 | Users can create/switch personas without cross-persona session leakage |
| T2.1.4 | Implement mandatory recovery phrase onboarding step | E2 / S2.1 | M | Web | T2.1.2 | Onboarding cannot complete until recovery phrase confirmation passes |
| T2.2.1 | Add invite token create/redeem flow (one-time or multi-use + optional expiration/max-uses) | E2 / S2.2 | L | API | T2.1.1 | Server owner can issue invite modes with optional expiration/max-uses, including non-expiring multi-use links; role/channel scoped grants are rejected in MVP |
| T2.3.1 | Implement nonce challenge-signature auth + session revoke endpoint | E2 / S2.4 | M | API | T2.1.1 | Auth succeeds only with valid signature; nonce is single-use with TTL/replay rejection; sessions are revocable |
| T2.4.1 | Add node fingerprint verification in join flow + security tests | E2 / S2.3 | M | API | T2.2.1, T2.3.1 | Client fails closed on fingerprint mismatch and tests cover replay/invalid token/exhausted invite cases |

## Task Touchpoints and Validation Gates

| Task | Target touchpoints | Validation |
|---|---|---|
| T1.1.1 | `apps/web/`, `services/api-rs/`, `services/realtime-rs/`, `infra/` | Workspace directories exist and project manifest(s) resolve without path errors |
| T1.1.2 | `infra/docker-compose.yml` | `docker compose up -d` and service health checks return healthy for Postgres, Redis, object storage, coturn |
| T1.1.3 | `scripts/`, `Makefile` or equivalent task runner | `setup`, `run`, and `test` commands execute from clean checkout |
| T1.2.1 | `.github/workflows/` | CI run fails on intentional lint/test failure and passes on clean branch |
| T1.2.2 | `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml` | OpenAPI file exists, includes all six Iteration 1 endpoints, and is linked from board/plan |
| T1.3.1 | `.env.example`, runtime config module(s) | Invalid env value causes startup failure with actionable error message |
| T2.1.1 | API identity schema, migrations, identity handler/module | Register endpoint persists key and rejects invalid algorithm/key format |
| T2.1.2 | Web identity onboarding flow + local key storage utility | Create/import succeeds; invalid key path surfaces `identity_key_invalid` state |
| T2.1.3 | Persona selector UI + persona-scoped session storage | Persona switch preserves isolation; integration test shows no persona data bleed |
| T2.1.4 | Recovery phrase onboarding UI + confirmation step | User cannot proceed without phrase confirmation and receives deterministic recovery backup status |
| T2.2.1 | Invite API handlers + invite persistence model | Expired/exhausted/invalid invite cases return expected error codes; non-expiring multi-use token path is test-covered |
| T2.3.1 | Auth challenge/verify handlers + session store | Duplicate/expired nonce fails with `nonce_invalid`; valid signature yields session |
| T2.4.1 | Join flow verification path + security test suite | Fingerprint mismatch blocks join (no warning-only path) and logs no secret-bearing fields |

## Entry Criteria

- `T1.1.1` branch plan and directory ownership are agreed by Core.
- Development environment has Docker and required runtimes installed.
- `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml` exists before starting `T2.*` tasks.

## Exit Evidence

- Evidence pack includes startup output, CI run links, and contract reference checks.
- Security test output for nonce replay, fingerprint mismatch, and invite exhaustion is attached.
- Final demo notes include end-to-end identity join/auth scenario and session revoke verification.

## Evidence Ledger

| Task set | Evidence artifact path | Validator |
|---|---|---|
| T1.1.x-T1.3.1 | `evidence/iteration-01/foundation/` | startup + CI gate verification |
| T2.1.x-T2.4.1 | `evidence/iteration-01/identity-auth-invites/` | OpenAPI conformance + security integration suite |

## In Progress

| ID | Task | Status | Notes |
|---|---|---|---|
| None | - | Not started | Iteration board created; execution not started |

## Done

| ID | Task | Completed In | Notes |
|---|---|---|---|
| None | - | - | - |

## Suggested Sprint Sequencing

Week 1:

- T1.1.1 -> T1.1.2 -> T1.1.3
- Start T1.2.1 and T1.3.1 in parallel after workspace stabilizes
- Publish T1.2.2 before Week 2 starts

Week 2:

- T2.1.1
- T2.1.2 and T2.1.3 (start once API contract is stable)
- T2.1.4
- T2.2.1

Week 3:

- T2.3.1
- T2.4.1
- Stabilization, test debt cleanup, iteration demo

## Iteration 1 Exit Checklist

- Local setup from clean machine in <= 20 minutes.
- CI checks green for Rust and web projects.
- Key identity join/auth flow working end-to-end.
- Multi-persona switching works with isolated sessions and settings.
- Invite creation and redeem flow working with mode and expiration checks.
- Recovery phrase onboarding is mandatory and validated.
- Session revoke working in UI and API.
- Security baseline checks complete (challenge-signature auth + token constraints + fingerprint verification).

## Execution Notes

- Keep PRs scoped to one task ID where possible.
- Tag commits/PR titles with task IDs (`T1.1.1`, etc.) for traceability.
- Freeze identity/invite API contracts before frontend polish using the Iteration 1 OpenAPI baseline in `docs/product/01-mvp-plan.md`.
- Apply navigation and screen hierarchy decisions from `docs/product/07-ui-navigation-spec.md` for all new UI surfaces.

## UI State Tables (Authority for MVP Iteration 1)

### Identity Create/Import (`T2.1.2`)

| State | Trigger | UI Behavior | Recovery Action |
|---|---|---|---|
| `identity_create_success` | Keypair generated and persisted | Success state and continue CTA | Proceed to next onboarding step |
| `identity_import_success` | Valid key import and persistence | Success state and continue CTA | Proceed to next onboarding step |
| `identity_key_invalid` | Invalid key format or verification failure | Inline error with guidance | Retry import with valid key material |
| `identity_storage_failed` | Local encrypted storage write failure | Blocking error state | Retry write; show fallback instructions |

### Join/Auth Errors (`T2.4.1`, `T2.3.1`)

| State | Trigger | UI Behavior | Recovery Action |
|---|---|---|---|
| `fingerprint_mismatch` | Node fingerprint does not match invite metadata | Blocking warning with explicit mismatch details | Abort join or retry with trusted invite |
| `invite_exhausted` | Invite max-uses reached | Blocking error with usage reason | Request a new invite from server owner |
| `invite_expired` | Invite expiration passed | Blocking error with expiration info | Request a new invite |
| `nonce_invalid` | Nonce expired/duplicate/invalid | Auth error state with retry option | Start a new auth challenge |
| `signature_invalid` | Signature verification failure | Auth error state without sensitive detail | Re-sign challenge and retry |

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
- `docs/reference/glossary.md`
