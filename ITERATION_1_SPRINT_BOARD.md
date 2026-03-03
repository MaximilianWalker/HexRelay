# HexRelay Iteration 1 Sprint Board

Scope: Iteration 1 (Weeks 1-3) from `MVP_PLAN.md`.

Goals:

- Stand up a working local development stack.
- Ship portable identity, invite auth, and session security.
- Lock CI quality gates and environment config discipline.

Effort scale:

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
| T1.3.1 | Add env schema validation and config templates | E1 / S1.3 | M | Platform | T1.1.1 | Invalid env values fail fast at startup with actionable errors |
| T2.1.1 | Implement key identity schema + key registration endpoints | E2 / S2.1 | XL | API | T1.1.1, T1.1.2 | Identity keys can be registered and validated with tests |
| T2.1.2 | Build client key generation/import + secure local key storage | E2 / S2.1 | L | Web | T2.1.1 | User can create or import identity and keep key material encrypted locally |
| T2.2.1 | Add invite token create/redeem flow (one-time or multi-use + expiration) | E2 / S2.2 | L | API | T2.1.1 | Server owner can issue invite modes with expiration and clients can redeem valid tokens |
| T2.3.1 | Implement nonce challenge-signature auth + session revoke endpoint | E2 / S2.4 | M | API | T2.1.1 | Auth succeeds only with valid signature and sessions are revocable |
| T2.4.1 | Add node fingerprint verification in join flow + security tests | E2 / S2.3 | M | API/Web | T2.2.1, T2.3.1 | Client warns/fails on fingerprint mismatch and tests cover replay/invalid token cases |

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

Week 2:

- T2.1.1
- T2.1.2 (start once API contract is stable)
- T2.2.1

Week 3:

- T2.3.1
- T2.4.1
- Stabilization, test debt cleanup, iteration demo

## Iteration 1 Exit Checklist

- Local setup from clean machine in <= 20 minutes.
- CI checks green for Rust and web projects.
- Key identity join/auth flow working end-to-end.
- Invite creation and redeem flow working with mode and expiration checks.
- Session revoke working in UI and API.
- Security baseline checks complete (challenge-signature auth + token constraints + fingerprint verification).

## Execution Notes

- Keep PRs scoped to one task ID where possible.
- Tag commits/PR titles with task IDs (`T1.1.1`, etc.) for traceability.
- Freeze identity/invite API contracts before frontend polish.
