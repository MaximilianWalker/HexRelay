# HexRelay Iteration Log

## Document Metadata

- Doc ID: iteration-log
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-10
- Source of truth: `docs/planning/05-iteration-log.md`

## Quick Context

- Primary edit location for project-level delivery changes across iterations.
- Do not duplicate sprint task detail here; link to iteration boards when needed.
- Latest meaningful change: 2026-03-10 readiness revalidation pass added runtime safeguard hardening and correction-log governance to prevent repeated audit loops.

## Purpose

- Capture project-level delivery changes that do not fit cleanly into a single sprint board update.
- Keep an auditable history of scope, sequencing, and status decisions.

## Entry Format

- Date (UTC)
- Area affected
- Change summary
- Rationale
- Linked docs updated

## Log Entries

### 2026-03-10 (readiness revalidation pass: recurrence prevention and runtime safeguard hardening)

- Area affected: Readiness governance, runtime startup resilience, websocket abuse controls, and evidence traceability.
- Change summary:
  - Added readiness correction governance policy in `AGENTS.md` and introduced `docs/operations/readiness-corrections-log.md` as the recurring-finding authority.
  - Hardened realtime websocket ingress: binary frames now share message-rate limits, handshake rejections return machine-readable error envelopes, and connect limit keying falls back to peer address when proxy headers are not trusted.
  - Added realtime numeric config guardrails for zero/degenerate values and minimum inbound payload thresholds.
  - Moved API and realtime tracing initialization before config parse to ensure startup failures are observable in logs.
  - Improved DB-backed API test behavior to use deterministic local default DB URL with explicit skip reason outside CI.
  - Expanded runtime REST contract to include implemented servers/contacts/friends endpoints and health probe.
  - Strengthened evidence provenance requirements (`commit_sha`, `pr_number`/`run_id`, `generated_at_utc`) across testing/evidence docs.
- Rationale:
  - Eliminate repeated readiness-audit rediscovery loops while increasing operational confidence and traceability for future feature work.

### 2026-03-05 (quality tightening pass: fail-closed controls and evidence completeness)

- Area affected: Realtime ingress trust boundary, API limiter resilience semantics, contact directory correctness, and CI evidence completeness.
- Change summary:
  - Tightened realtime websocket policy to require allowed `Origin`; missing origin now rejected.
  - Updated API distributed rate-limit behavior to fail closed when DB-backed limiter is unavailable in DB runtime mode.
  - Removed silent DB/decode fallback in contacts directory path; DB errors now surface as explicit API errors.
  - Raised Rust coverage CI threshold from 55% to 65% for stronger baseline regression confidence.
  - Expanded CI evidence artifacts with machine-readable `summary.json`, SHA256 file hashes, and coverage summary capture; added evidence index doc.
- Rationale:
  - Ensure implemented hardening improves real runtime quality under incident and abuse conditions rather than masking failures.

### 2026-03-05 (auth/key-management priority clarification)

- Area affected: Product security hardening scope and MVP prioritization.
- Change summary:
  - Recorded that passphrase-gated local key unlock remains optional hardening and is not an MVP priority.
  - Confirmed current MVP baseline remains cookie-first auth transport + CSRF + runtime abuse controls without mandatory passphrase UX.
- Rationale:
  - Preserve low-friction onboarding and avoid premature UX/security coupling while core functionality is still under active delivery.

### 2026-03-05 (readiness controls pass: security gates, evidence automation, distributed limiting, realtime guardrails)

- Area affected: CI security posture, release evidence quality, API abuse control scalability, realtime resilience, and API handler maintainability.
- Change summary:
  - Added CI security automation gates for Rust dependencies (`cargo audit`), web dependencies (`npm audit --omit=dev --audit-level=high`), and static analysis (`semgrep`).
  - Added deterministic CI evidence collection script and integration-smoke artifact upload under `evidence/ci/<run_id>/`.
  - Added DB-backed distributed API rate limiting counters (`rate_limit_counters`) to preserve abuse-control behavior across multi-instance API deployments sharing Postgres.
  - Added relational FK constraints for `sessions`, `auth_challenges`, and `friend_requests` against `identity_keys` to tighten persistence integrity.
  - Added realtime websocket guardrails: per-identity connection cap, inbound message-size cap, and per-identity message-rate cap.
  - Added realtime websocket `Origin` allowlist enforcement for browser-originated upgrades.
  - Added Rust coverage threshold gate in CI to provide quantitative backend test-confidence enforcement.
  - Continued handler decomposition by extracting directory/list endpoints into dedicated `directory_handlers` module.
- Rationale:
  - Improve confidence on substantive remaining quality risks while preserving local-first desktop defaults and enabling stronger dedicated-server safety under active development.

### 2026-03-05 (auth transport migration to HttpOnly cookie + CSRF)

- Area affected: Runtime auth transport across API, web client, realtime validation path, and runtime contracts.
- Change summary:
  - Switched runtime web auth transport from JS-managed bearer token usage to HttpOnly session cookie (`hexrelay_session`).
  - Added double-submit CSRF enforcement (`hexrelay_csrf` cookie + `x-csrf-token` header) for authenticated mutation routes.
  - Updated web API calls to `credentials: include` and removed auth token plumbing from page-level calls.
  - Updated realtime session validation forwarding to support cookie-authenticated websocket handshakes.
  - Updated runtime OpenAPI contracts and runbook auth language to reflect cookie-first transport.
  - Supersedes prior runtime bearer-token transport notes in historical entries below.
- Rationale:
  - Reduce token exfiltration risk from browser script-accessible storage while keeping runtime auth/session behavior explicit and testable.

### 2026-03-04 (security and hygiene hardening: token rotation, rate limiting, and runtime contract cleanup)

- Area affected: Auth/session security, abuse controls, runtime contract governance, and dead/legacy runtime path cleanup.
- Change summary:
  - Added versioned bearer token format (`HEXTOKEN_V1`) with signing key ID support and keyring-based token validation.
  - Added API rate limits for auth challenge/verify and invite create/redeem paths.
  - Added realtime websocket connect rate limiting.
  - Removed non-test runtime fallback behavior for identity/auth/invite/session critical storage paths; runtime now requires DB-backed authority for these flows.
  - Promoted runtime REST contract authority to `docs/contracts/runtime-rest-v1.openapi.yaml` and retained legacy Iteration-1 contract path as compatibility alias.
  - Added confidence-hardening evidence artifact baseline under `evidence/iteration-01/confidence-hardening/`.
- Rationale:
  - Reduce attack surface and runtime drift before additional feature expansion, while cleaning legacy authority naming and dead fallback runtime branches.
- Linked docs updated:
  - `docs/contracts/runtime-rest-v1.openapi.yaml`
  - `docs/contracts/README.md`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/contracts/mvp-rest-v1.openapi.yaml`
  - `docs/README.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`
  - `evidence/iteration-01/confidence-hardening/2026-03-04-quality-validation.md`
  - `services/api-rs/src/session_token.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/invite_handlers.rs`
  - `services/api-rs/src/auth.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `services/realtime-rs/src/config.rs`
  - `services/realtime-rs/src/state.rs`

### 2026-03-04 (confidence hardening: independent-audit blocker closure)

- Area affected: Readiness blocker remediation
- Change summary:
  - Split current-runtime contracts and target-state model contracts by adding a contracts index and runtime realtime contract artifact.
  - Updated docs index routing so runtime behavior references runtime contracts while roadmap contracts remain explicitly model-only.
  - Added missing metadata/Quick Context blocks for runtime ADR and crypto contract/checklist docs, and linked the crypto checklist from testing index.
  - Updated sprint board metadata statuses to match active execution state for Iteration 1 and 2.
  - Reconciled stale Iteration 1 board notes that still described identity/session persistence as in-memory.
  - Added dedicated-server restore evidence contract requirements in runbook.
  - Added license artifact for documented AGPL baseline.
  - Reduced session token exposure persistence by storing access tokens in `sessionStorage` while keeping session metadata in local storage.
  - Enforced DB test confidence in CI by failing when `API_DATABASE_URL` is missing under CI context.
- Rationale:
  - Resolve independent hard-pass blockers that were not stylistic and materially affected confidence.
- Linked docs updated:
  - `docs/contracts/README.md`
  - `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`
  - `docs/contracts/crypto-profile-v1.md`
  - `docs/contracts/mvp-rest-v1.openapi.yaml`
  - `docs/contracts/realtime-events-v1.asyncapi.yaml`
  - `docs/architecture/adr-0002-runtime-deployment-modes.md`
  - `docs/README.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/operations/contributor-guide.md`
  - `docs/testing/README.md`
  - `docs/testing/crypto-conformance-checklist.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `LICENSE`
  - `apps/web/lib/sessions.ts`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/db.rs`

### 2026-03-04 (confidence hardening: realtime contract and transport safety)

- Area affected: Realtime trust boundary and contract conformance
- Change summary:
  - Hardened realtime config to enforce valid API URL scheme and require HTTPS for non-loopback API upstreams.
  - Added strict realtime HTTP client timeout/connect-timeout defaults for auth validation calls.
  - Replaced websocket text echo behavior with structured event-envelope routing for call signaling event types.
  - Enforced realtime sender identity binding by validating `from_user_id` against authenticated session identity before accepting signaling payloads.
  - Added realtime contract tests for event version validation, unsupported event handling, malformed payloads, and websocket roundtrip envelope shape.
  - Added negative integration test for websocket auth flow when API upstream is unreachable.
  - Added DB migration backfill test for invite plaintext-token hashing and removed plaintext fallback from invite redeem queries.
  - Added web unit tests for secure-store provider failure fallback and recovery phrase derivation stability.
  - Aligned product stack wording to current HMAC bearer token model (removed JWT phrasing drift).
- Rationale:
  - Reduce auth-gate failure ambiguity and establish deterministic realtime event contract behavior before broader fanout feature work.
- Linked docs updated:
  - `services/realtime-rs/src/config.rs`
  - `services/realtime-rs/src/state.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `services/realtime-rs/Cargo.toml`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/migrations/0007_invites_hash_backfill.sql`
  - `services/api-rs/src/invite_handlers.rs`
  - `apps/web/lib/secure-store.test.ts`
  - `apps/web/lib/recovery.test.ts`
  - `docs/product/01-mvp-plan.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (confidence hardening phase 1-2 kickoff)

- Area affected: Deployment clarity and security baseline
- Change summary:
  - Added explicit source-of-truth authority notes in infra and service READMEs to reduce runtime/deployment guidance drift.
  - Expanded MVP runbook with concrete dedicated-server startup order, TLS boundary assumptions, and restart validation checks.
  - Added runtime term mapping in glossary and linked PRD/plan runtime sections to glossary authority.
  - Replaced static onboarding recovery phrase with generated per-session phrase flow.
  - Introduced secure-store abstraction for private key encryption materials (provider-backed when available, session fallback otherwise).
  - Hardened invite storage by persisting hashed invite tokens instead of plaintext tokens for new records (with backward-compatible redeem matching).
  - Aligned long-range REST contract bearer token format wording to current token model (`HEXTOKEN`).
- Rationale:
  - Raise readiness confidence before further feature expansion by tightening both contributor-operational clarity and critical auth/privacy handling paths.
- Linked docs updated:
  - `infra/README.md`
  - `services/api-rs/README.md`
  - `services/realtime-rs/README.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/operations/contributor-guide.md`
  - `docs/reference/glossary.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/contracts/mvp-rest-v1.openapi.yaml`
  - `apps/web/lib/secure-store.ts`
  - `apps/web/lib/sessions.ts`
  - `apps/web/lib/recovery.ts`
  - `apps/web/app/onboarding/recovery/page.tsx`
  - `services/api-rs/src/invite_handlers.rs`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (documentation alignment: runtime/deployment model)

- Area affected: Product and architecture context clarity
- Change summary:
  - Locked and documented primary runtime as bundled desktop local-first mode.
  - Added explicit local UI launch options: embedded desktop shell or local-browser access on localhost.
  - Documented dedicated server mode as supported optional deployment path.
  - Added ADR-0002 for runtime/deployment modes and aligned README, product, operations, and service docs.
- Rationale:
  - Remove ambiguity about browser-only hosted assumptions and keep implementation decisions aligned with off-grid desktop goals.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/architecture/README.md`
  - `docs/architecture/adr-0002-runtime-deployment-modes.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/reference/glossary.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/operations/contributor-guide.md`
  - `apps/web/README.md`
  - `services/api-rs/README.md`
  - `services/realtime-rs/README.md`
  - `AGENTS.md`

### 2026-03-04 (execution batch: contacts optimism, invite UX, cross-service smoke gate)

- Area affected: Iteration 2 delivery velocity and integration safety
- Change summary:
  - Added optimistic friend-request UX behavior in Contacts hub with rollback/error messaging and action busy states for send/accept/decline.
  - Added in-app invite create/redeem controls to Contacts hub to execute invite workflows outside onboarding.
  - Added cross-service smoke path for `web -> api -> realtime` with CI `integration-smoke` job (Postgres-backed services + websocket auth handshake validation).
  - Added smoke runner script (`apps/web/scripts/e2e-smoke.mjs`) and web package command `e2e:smoke`.
  - Started API handler modularization by extracting invite handlers into `services/api-rs/src/invite_handlers.rs` and re-exporting via `services/api-rs/src/handlers.rs`.
- Rationale:
  - Continue feature delivery while preserving confidence through real cross-service validation and reducing handler-file growth pressure.
- Linked docs updated:
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/scripts/e2e-smoke.mjs`
  - `apps/web/package.json`
  - `.github/workflows/ci.yml`
  - `services/api-rs/src/invite_handlers.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `docs/planning/05-iteration-log.md`
  - `docs/planning/iterations/02-sprint-board.md`

### 2026-03-04 (readiness uplift: persistence and CI coverage gates)

- Area affected: Future-development readiness hardening
- Change summary:
  - Added DB-backed identity-key persistence with new migration and API handler DB paths for registration/challenge/verify identity lookup.
  - Added DB-backed auth-challenge and invite durability (`auth_challenges`, `invites`) with restart-safe verification/redeem test coverage.
  - Aligned auth challenge TTL to 60 seconds (`CHALLENGE_TTL_SECONDS = 60`) to match crypto profile expectations.
  - Made API session signing key mandatory from environment to remove insecure fallback-key behavior.
  - Added realtime websocket-gate integration tests (authorized upgrade + unauthorized rejection) and expanded web API transport tests.
  - Raised web coverage thresholds and enforced coverage execution in CI via `test:coverage`.
  - Updated CI to provision Postgres for Rust checks and pass API DB/signing env vars so DB integration paths execute under CI.
- Rationale:
  - Raise confidence from "good" to "high" by ensuring critical auth/persistence paths are both enforced and continuously validated in CI.
- Linked docs updated:
  - `services/api-rs/migrations/0004_identity_keys.sql`
  - `services/api-rs/migrations/0005_auth_challenges.sql`
  - `services/api-rs/migrations/0006_invites.sql`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/lib.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `apps/web/lib/api.test.ts`
  - `apps/web/vitest.config.ts`
  - `apps/web/package.json`
  - `apps/web/package-lock.json`
  - `.github/workflows/ci.yml`
  - `docs/planning/05-iteration-log.md`
  - `docs/planning/iterations/02-sprint-board.md`

### 2026-03-04 (stabilization follow-up: replay race, migration safety, contract sync)

- Area affected: Auth/session correctness and persistence safety gates
- Change summary:
  - Made auth challenge consumption atomic in verify flow (challenge removed under write lock before signature verification) to eliminate replay race window.
  - Hardened migration lock lifecycle with guaranteed unlock attempt after migration execution path returns.
  - Added DB-backed integration tests for session validate/revoke lifecycle and migration checksum mismatch detection/lock release behavior.
  - Added concurrent replay test ensuring only one verify succeeds for duplicate challenge verification attempts.
  - Updated Iteration 1 OpenAPI contract to include bearer-auth requirements, session validate endpoint, and `access_token` in auth verify response.
- Rationale:
  - Complete mandatory hardening preconditions so future Iteration 2 feature work builds on deterministic auth and migration invariants.
- Linked docs updated:
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/lib.rs`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/planning/05-iteration-log.md`
  - `docs/planning/iterations/02-sprint-board.md`

### 2026-03-04 (session-token enforcement and migration checksum hardening)

- Area affected: Cross-cutting auth/session and persistence integrity
- Change summary:
  - Added signed bearer session token flow end-to-end: API issues `access_token` on verify and web stores/uses it for protected API calls.
  - Tightened API auth-sensitive routes by requiring authenticated context for revoke and list endpoints, with session-id match enforcement for revoke.
  - Updated API CORS allow-headers to include `Authorization` for browser preflight compatibility.
  - Added DB `sessions` migration plus migration checksum tracking and advisory-lock guarded migration execution.
  - Updated realtime API-validation bridge to require and forward `Authorization` when checking websocket session validity.
  - Rewired web hubs/onboarding/home to consume `access_token` consistently for servers/contacts/friend-request and session revoke paths.
- Rationale:
  - Remove header-forgery-prone session-only transport as the primary path and align runtime auth to signed token + server-side validation.
- Linked docs updated:
  - `services/api-rs/migrations/0003_sessions.sql`
  - `services/api-rs/src/session_token.rs`
  - `services/api-rs/src/auth.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/Cargo.toml`
  - `services/realtime-rs/src/handlers.rs`
  - `apps/web/lib/sessions.ts`
  - `apps/web/lib/api.ts`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/home/page.tsx`
  - `Cargo.lock`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (quality hardening batch: auth, cors, realtime gate)

- Area affected: Cross-cutting quality/security hardening
- Change summary:
  - Added identity registration conflict guard to prevent silent key overwrite for existing identities.
  - Added `GET /v1/auth/sessions/validate` and reused centralized `AuthSession` extractor for server-side session-bound auth context.
  - Restricted API CORS from wildcard to env-driven explicit allowlist (`API_ALLOWED_ORIGINS`).
  - Hardened friend-request handlers to require database pool in non-test runtime and keep in-memory path only for tests.
  - Added realtime websocket auth gate by validating `x-session-id` against API before upgrade.
  - Improved frontend hubs (`/servers`, `/contacts`) with explicit network error catch/finalization paths.
- Rationale:
  - Close immediate trust-boundary and operational safety gaps before additional feature expansion.
- Linked docs updated:
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/auth.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/errors.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/main.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/.env.example`
  - `services/realtime-rs/src/state.rs`
  - `services/realtime-rs/src/config.rs`
  - `services/realtime-rs/src/app.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `services/realtime-rs/src/main.rs`
  - `services/realtime-rs/.env.example`
  - `services/realtime-rs/Cargo.toml`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (migration governance and transition-matrix hardening)

- Area affected: Iteration 2 social graph hardening (`T3.1.1`)
- Change summary:
  - Added versioned migration runner with tracked `schema_migrations` table and explicit migration files for friend-request schema/indexes.
  - Added centralized Axum `AuthSession` extractor and rewired friend-request handlers to use shared auth context instead of duplicated header parsing.
  - Added strict transition matrix behavior for friend requests: pending-only mutations, requester-only cancel, target-only accept/decline.
  - Added idempotent semantics for repeated same terminal action and `409 transition_invalid` for conflicting non-pending transitions.
  - Extended tests for missing session auth, wrong actor rejection, cancel flow, and conflicting transition rejection.
- Rationale:
  - Enforce durable schema evolution and deterministic social-graph mutation rules before scaling social features.
- Linked docs updated:
  - `services/api-rs/src/auth.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/errors.rs`
  - `services/api-rs/migrations/0001_friend_requests.sql`
  - `services/api-rs/migrations/0002_friend_requests_transition_index.sql`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (friend request Postgres persistence hardening)

- Area affected: Iteration 2 social graph persistence (`T3.1.1`)
- Change summary:
  - Added `sqlx` Postgres integration in `api-rs` with startup schema preparation for `friend_requests`.
  - Added DB-backed create/list/accept/decline friend-request handlers with fallback to in-memory state for non-DB contexts.
  - Added centralized Axum auth extractor (`AuthSession`) for session-bound actor enforcement via `x-session-id` and server-side session lookup.
  - Added pending-only transition guards so accept/decline cannot mutate non-pending requests or unauthorized actors.
  - Added runtime `API_DATABASE_URL` config and updated service env template.
  - Preserved and revalidated full Rust/Web test suite after dependency/version compatibility pinning.
- Rationale:
  - Move friend-request lifecycle off volatile in-memory storage to a durable persistence path before expanding social graph features.
- Linked docs updated:
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/main.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/Cargo.toml`
  - `services/api-rs/.env.example`
  - `Cargo.lock`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (onboarding scope simplification)

- Area affected: Onboarding UX scope (`T2.1.4`)
- Change summary:
  - Removed server join/contact invite actions from onboarding access step.
  - Converted access step into completion/handoff screen that routes users into the main app hubs for join/invite flows.
- Rationale:
  - Reduce onboarding complexity and user confusion by keeping onboarding focused on identity and recovery only.
- Linked docs updated:
  - `apps/web/app/onboarding/access/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (friend-request API baseline and contacts request actions)

- Area affected: Iteration 2 social graph bootstrap (`T3.1.1`, `T3.1.2`)
- Change summary:
  - Implemented friend-request endpoints in `api-rs`: create/list plus accept/decline transitions.
  - Added query validation and in-memory state tracking for pending request lifecycle.
  - Added API tests for create/list and accept/decline behavior.
  - Wired Contacts hub to live friend-request endpoints and added send/accept/decline UI actions.
  - Added API-backed Servers/Contacts read endpoints and dynamic server workspace route scaffold for route continuity.
- Rationale:
  - Start Iteration 2 social graph execution on top of already stable identity/auth/invite primitives while keeping implementation deterministic and test-covered.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/src/lib.rs`
  - `apps/web/lib/api.ts`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/servers/[serverId]/page.tsx`
  - `apps/web/app/surfaces.module.css`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (servers and contacts API-backed hub wiring)

- Area affected: Iteration 1 web/backend integration (`T2.1.3` support)
- Change summary:
  - Added API read endpoints `GET /v1/servers` and `GET /v1/contacts` with deterministic query filtering.
  - Added backend tests covering server/contact list filtering paths.
  - Rewired web Servers and Contacts routes to call live API endpoints instead of local in-file datasets.
  - Added dynamic server workspace route scaffold at `/servers/[serverId]` and linked server cards to workspace route navigation.
  - Preserved screen-state mapping (`loading`, `error`, `empty`, `search_no_results`, request states) while changing data source to backend.
- Rationale:
  - Reduce placeholder logic and align hub surfaces with real API contracts ahead of friend/guild persistence work.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/lib.rs`
  - `apps/web/lib/api.ts`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (hub state interactivity pass)

- Area affected: Iteration 1 web hub/state execution (`T2.1.3` support)
- Change summary:
  - Upgraded Servers and Contacts routes to client-interactive hubs with search and filter toggles.
  - Added explicit screen-state rendering outputs (`empty`, `search_no_results`, `friend_request_pending`, `friend_request_inbound`, `ready`) in hub surfaces.
  - Added Settings DM inbound policy persistence (`friends_only`, `same_server`, `anyone`) with per-device local preference storage.
- Rationale:
  - Move hub pages from static placeholders to stateful surfaces aligned with MVP screen-state spec before deeper backend query wiring.
- Linked docs updated:
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/settings/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (workspace shell and top-level navigation surfaces)

- Area affected: Iteration 1 web navigation execution (`T2.1.3`, `T2.1.4` support)
- Change summary:
  - Added shared workspace shell component with top-level navigation (`Home`, `Servers`, `Contacts`, `Settings`) and mobile tab switcher.
  - Added server dual-navigation baseline affordances: collapsible sidebar preference and top tab strip.
  - Added initial route surfaces for `/servers`, `/contacts`, and `/settings` aligned with hub/filter state requirements.
  - Migrated `/home` to run inside shared shell while preserving persona/session controls.
- Rationale:
  - Align executable UI structure with navigation spec so subsequent feature work lands on stable route/layout primitives.
- Linked docs updated:
  - `apps/web/components/workspace-shell.tsx`
  - `apps/web/components/workspace-shell.module.css`
  - `apps/web/app/home/page.tsx`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/settings/page.tsx`
  - `apps/web/app/surfaces.module.css`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (key-at-rest encryption and persona session revoke wiring)

- Area affected: Iteration 1 web security and session lifecycle execution (`T2.1.2`, `T2.1.3`, `T2.3.1`)
- Change summary:
  - Replaced plain localStorage private-key persistence with persona-scoped AES-GCM encrypted storage.
  - Added Home persona remove action and switch-time session revoke integration using `POST /v1/auth/sessions/revoke`.
  - Added persona cleanup paths to remove encrypted key/session records on persona deletion.
  - Added lightweight onboarding/home telemetry event tracking for API flow stages and failures.
- Rationale:
  - Tighten local key handling and enforce deterministic session lifecycle behavior during persona transitions.
- Linked docs updated:
  - `apps/web/lib/sessions.ts`
  - `apps/web/lib/personas.ts`
  - `apps/web/lib/api.ts`
  - `apps/web/lib/telemetry.ts`
  - `apps/web/app/home/page.tsx`
  - `apps/web/app/home/home.module.css`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/onboarding/access/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (identity auth wiring and invite create UX integration)

- Area affected: Iteration 1 onboarding API integration (`T2.1.2`, `T2.2.1`, `T2.3.1`)
- Change summary:
  - Wired identity onboarding to live API flow: register identity key -> challenge issue -> challenge verify.
  - Added client crypto utilities for ed25519 key generation/import parsing and nonce signature generation.
  - Added persona-scoped local session/private-key storage utilities and stored auth session on successful verify.
  - Added onboarding access action to create test invites via live `POST /v1/invites` before redemption.
  - Extended web API client module to cover identity/auth/invite endpoints.
- Rationale:
  - Replace onboarding placeholders with executable integration against implemented Iteration 1 API primitives.
- Linked docs updated:
  - `apps/web/lib/api.ts`
  - `apps/web/lib/crypto.ts`
  - `apps/web/lib/sessions.ts`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/onboarding/access/page.tsx`
  - `apps/web/package.json`
  - `apps/web/package-lock.json`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (persona isolation scaffold)

- Area affected: Iteration 1 web identity execution (`T2.1.3`)
- Change summary:
  - Added browser-local persona storage utilities with active-persona tracking.
  - Wired onboarding identity step to persist/select persona before moving to recovery.
  - Replaced `/home` placeholder with persona management and switching surface showing active-session context.
- Rationale:
  - Establish deterministic client-side persona/session isolation baseline before deeper auth/session integration.
- Linked docs updated:
  - `apps/web/lib/personas.ts`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/home/page.tsx`
  - `apps/web/app/home/home.module.css`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (node fingerprint verification and onboarding API wiring)

- Area affected: Iteration 1 join-flow security and web onboarding integration (`T2.4.1`)
- Change summary:
  - Added invite-bound node fingerprint enforcement in `api-rs` redeem flow; mismatched fingerprint now fails with `fingerprint_mismatch`.
  - Added CORS middleware to API router so web onboarding can call local API endpoints in dev.
  - Added `API_NODE_FINGERPRINT` runtime config and threaded value into application state.
  - Added API tests for fingerprint mismatch rejection and updated invite redeem tests to include expected node fingerprint.
  - Wired onboarding access screen to live `POST /v1/invites/redeem` calls and mapped API error codes (`invite_invalid`, `invite_expired`, `invite_exhausted`, `fingerprint_mismatch`).
- Rationale:
  - Enforce fail-closed join verification at API boundary and remove placeholder token simulation from onboarding.
- Linked docs updated:
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/main.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/Cargo.toml`
  - `services/api-rs/.env.example`
  - `Cargo.lock`
  - `apps/web/lib/api.ts`
  - `apps/web/app/onboarding/access/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (onboarding flow shell implementation)

- Area affected: Iteration 1 onboarding web execution (`T2.1.2`, `T2.1.4`)
- Change summary:
  - Replaced starter web screen with route-based onboarding flow: `/onboarding/identity`, `/onboarding/recovery`, `/onboarding/access`.
  - Added identity create/import UX shell with validation-state feedback and persona labeling scaffold.
  - Added mandatory recovery checkpoint UX requiring phrase word confirmation before progression.
  - Added access choice UX for server invite, direct contact invite, or skip path plus `/home` post-onboarding placeholder.
  - Updated global web styling baseline and font stack for a dedicated product visual direction.
- Rationale:
  - Move from scaffolding UI to executable onboarding flow aligned with Iteration 1 product requirements.
- Linked docs updated:
  - `apps/web/app/page.tsx`
  - `apps/web/app/layout.tsx`
  - `apps/web/app/globals.css`
  - `apps/web/app/onboarding/onboarding.module.css`
  - `apps/web/app/onboarding/page.tsx`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/onboarding/recovery/page.tsx`
  - `apps/web/app/onboarding/access/page.tsx`
  - `apps/web/app/home/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (invite create/redeem baseline)

- Area affected: Iteration 1 invite execution (`T2.2.1`)
- Change summary:
  - Implemented `POST /v1/invites` and `POST /v1/invites/redeem` in `services/api-rs`.
  - Added invite mode/expiry/max-uses validation including one-time invite max-use enforcement.
  - Added deterministic invalid, expired, and exhausted invite behavior with explicit error codes.
  - Added API tests for multi-use redeem success, one-time exhaustion, and expired invite rejection.
  - Updated Iteration 1 OpenAPI with invite create/redeem response schemas.
- Rationale:
  - Complete baseline invite lifecycle behavior needed for Iteration 1 join/auth flow dependencies.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (auth verify and session revoke baseline)

- Area affected: Iteration 1 auth execution (`T2.3.1`)
- Change summary:
  - Implemented `POST /v1/auth/verify` with nonce lookup/expiry checks, ed25519 signature verification, single-use challenge consumption, and in-memory session issuance.
  - Implemented `POST /v1/auth/sessions/revoke` with deterministic invalid-session rejection.
  - Added API tests covering verify/revoke success path and invalid signature rejection.
  - Updated Iteration 1 OpenAPI to include `AuthVerifyResponse` and explicit `400/401` verify outcomes.
- Rationale:
  - Complete the core challenge-signature auth loop so session lifecycle behavior is executable before moving to invite/join hardening.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/src/errors.rs`
  - `services/api-rs/Cargo.toml`
  - `Cargo.lock`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (auth challenge endpoint baseline)

- Area affected: Iteration 1 auth bootstrap execution (`T2.3.1`)
- Change summary:
  - Implemented `POST /v1/auth/challenge` in `services/api-rs` with registered-identity enforcement and nonce challenge issuance.
  - Added in-memory challenge store to API state and modularized handler wiring to include auth challenge routing.
  - Added API tests for challenge issuance (registered identity) and unknown identity rejection.
  - Updated Iteration 1 OpenAPI contract to include `AuthChallengeResponse` schema.
- Rationale:
  - Unblock signature-verify flow by providing deterministic challenge issuance behavior aligned to the Iteration 1 contract.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/Cargo.toml`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (iteration 1 identity endpoint start)

- Area affected: Iteration 1 `T2.1.1` execution progress
- Change summary:
  - Implemented `POST /v1/identity/keys/register` in `services/api-rs` with fail-fast validation for algorithm and public key format.
  - Added API tests covering success path and invalid algorithm/key rejection.
  - Aligned Iteration 1 OpenAPI error-code enum with identity registration validation errors.
  - Marked `T2.1.1` as in progress in the Iteration 1 board.
- Rationale:
  - Establish executable identity registration baseline before challenge/verify and invite flows.
- Linked docs updated:
  - `services/api-rs/src/main.rs`
  - `services/api-rs/Cargo.toml`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (iteration 1 quality and config gates)

- Area affected: Iteration 1 quality gate enforcement and configuration validation
- Change summary:
  - Hardened CI workflow to run active Rust/Web quality gates without scaffold-skip detection.
  - Added runtime environment validation for API and realtime services (`API_BIND`, `REALTIME_BIND`).
  - Added web environment schema validation for API and realtime endpoint URLs.
  - Added `.env.example` templates for `apps/web`, `services/api-rs`, and `services/realtime-rs`.
  - Marked `T1.2.1` and `T1.3.1` completed in the Iteration 1 board.
- Rationale:
  - Ensure invalid configuration fails fast and CI gates are enforceable before starting identity/auth implementation tasks.
- Linked docs updated:
  - `.github/workflows/ci.yml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `apps/web/.env.example`
  - `services/api-rs/.env.example`
  - `services/realtime-rs/.env.example`

### 2026-03-04 (iteration 1 foundation kickoff)

- Area affected: Iteration 1 execution tracking
- Change summary:
  - Marked `T1.1.1`, `T1.1.2`, and `T1.1.3` as complete in the Iteration 1 board.
  - Added one-command workspace flows via root npm scripts (`setup`, `run`, `test`).
  - Updated root getting-started guidance to reflect runnable scaffold bootstrap.
- Rationale:
  - Align task status with completed implementation bootstrap work before moving to `T1.2.x` and `T1.3.1`.
- Linked docs updated:
  - `README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `scripts/README.md`
  - `package.json`

### 2026-03-04 (development bootstrap execution)

- Area affected: Project development readiness and execution gates
- Change summary:
  - Initialized runnable web scaffold in `apps/web` with lint/test/build scripts.
  - Initialized Rust service scaffolds in `services/api-rs` and `services/realtime-rs` with workspace `Cargo.toml`.
  - Added local infra stack in `infra/` with compose, env defaults, and TURN configuration.
  - Added CI workflow in `.github/workflows/ci.yml` with Rust/Web quality gates.
  - Replaced placeholder workspace automation with executable scripts and `Makefile` targets.
  - Promoted dependency gates `D-001` to `D-007` to `ready` in dependency register.
- Rationale:
  - Move from planning-only to an executable baseline so Iteration 1 development can begin with enforceable quality gates.
- Linked docs updated:
  - `README.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (dm offline policy lock)

- Area affected: DM reliability semantics and UX expectations
- Change summary:
  - Locked MVP DM offline behavior to best-effort online delivery.
  - Added encrypted local outbox retry expectation to DM execution and verification docs.
  - Registered config default, risk, and decision entries for offline DM behavior.
- Rationale:
  - Preserve direct user-to-user DM transport without introducing server-side DM queues in MVP.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/reference/glossary.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (dm transport architecture correction)

- Area affected: Core messaging architecture and MVP execution tasks
- Change summary:
  - Corrected DM architecture to direct user-to-user transport with no guild/community server relay/storage.
  - Updated plan, PRD, Iteration 2 tasks, REST/realtime contracts, data lifecycle matrix, and verification matrix to match this model.
  - Removed server-ciphertext DM assumptions from execution and validation language.
- Rationale:
  - Align implementation docs with core product intent: server communities should not be DM transport intermediaries.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/contracts/mvp-rest-v1.openapi.yaml`
  - `docs/contracts/realtime-events-v1.asyncapi.yaml`
  - `docs/architecture/02-data-lifecycle-retention-replication.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (execution hardening deep-pass)

- Area affected: Full-MVP documentation precision and implementation readiness
- Change summary:
  - Added MVP REST contract coverage baseline for Iterations 2-4.
  - Added canonical screen-state spec and configuration defaults register.
  - Added architecture-level data lifecycle/retention/replication matrix.
  - Added MVP operations runbook and requirement-to-evidence verification matrix.
  - Added UI/flow state mappings and evidence ledgers to Iterations 1-4 boards.
  - Updated docs indexes to register new canonical artifacts.
- Rationale:
  - Reduce cross-team ambiguity during parallel implementation.
  - Make requirement -> task -> evidence trace deterministic.
- Linked docs updated:
  - `docs/contracts/mvp-rest-v1.openapi.yaml`
  - `docs/product/08-screen-state-spec.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/architecture/02-data-lifecycle-retention-replication.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/testing/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/product/README.md`
  - `docs/architecture/README.md`
  - `docs/operations/README.md`
  - `docs/planning/README.md`
  - `docs/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (privacy-first social policy lock)

- Area affected: MVP friend request and DM onboarding behavior
- Change summary:
  - Locked server-mediated friend request model for in-server contact flows.
  - Locked default privacy rule preventing raw key/profile-identifying data exposure before acceptance.
  - Added Iteration 2 tasks for mediated identity bootstrap release and DM inbound policy defaults/overrides.
  - Added risk and decision coverage for identity scraping prevention.
- Rationale:
  - Preserve user privacy by default while keeping server-assisted contact discovery usable.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/reference/glossary.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (server invite policy normalization)

- Area affected: MVP server onboarding policy and invite semantics
- Change summary:
  - Locked server invite policy to allow optional expiration and optional max-uses.
  - Explicitly allowed non-expiring multi-use invite links as an open-access pattern.
  - Updated Iteration 1 task acceptance and OpenAPI schema to cover optional invite policy fields.
  - Added clarification and decision entries for this policy.
- Rationale:
  - Keep invite-based architecture while supporting practical open-server behavior without separate join modes.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (direct user contact invite lock)

- Area affected: MVP social graph onboarding and direct user add flow
- Change summary:
  - Added direct user contact invite flow (expiring link + QR) to MVP plan and PRD.
  - Added Iteration 2 API/Web tasks for contact invite create/redeem and share/scan UX.
  - Extended requirement-to-task matrix with direct user invite coverage.
- Rationale:
  - Allow users to add each other directly without depending on global/shared-server discovery.
  - Align user add UX with invite-based mental model already used for server joins.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (post-MVP discovery roadmap lock)

- Area affected: Post-MVP product roadmap direction
- Change summary:
  - Locked post-MVP discovery strategy to hybrid mode.
  - Federation discovery remains supported, trusted-registry scopes are planned, and full P2P discovery is an optional later mode.
  - Updated plan, PRD, clarifications, and decisions register to reflect this direction.
- Rationale:
  - Preserve self-hosted usability and selective discoverability while keeping a clear path toward deeper decentralization.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (migration precedence decision lock)

- Area affected: Iteration 4 migration reconciliation policy
- Change summary:
  - Resolved `C-014` with canonical rule: user-signed profile data is authoritative for profile fields.
  - Locked server role to identity/security/membership enforcement, not profile-field authority.
  - Updated migration and profile authority wording in product plan, PRD, risk register, and Iteration 4 entry gate.
- Rationale:
  - Preserve user data ownership model while keeping server-side security and permission enforcement deterministic.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (clarification resolution and artifact lock)

- Area affected: Full-picture pre-MVP planning gates
- Change summary:
  - Resolved `C-012` by adding versioned realtime contract artifact `docs/contracts/realtime-events-v1.asyncapi.yaml`.
  - Resolved `C-013` by adding fixed KPI/SLO benchmark profile `docs/planning/kpi-slo-test-profile.md`.
  - Linked Iteration 2/3/4 gate language to resolved artifacts and clarification IDs.
  - Kept `C-014` open pending migration conflict precedence decision.
- Rationale:
  - Remove remaining planning ambiguity for realtime contracts and KPI/SLO evidence.
  - Preserve one explicit final decision gate before Iteration 4 migration sign-off.
- Linked docs updated:
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/contracts/realtime-events-v1.asyncapi.yaml`
  - `docs/planning/kpi-slo-test-profile.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/README.md`
  - `docs/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (full-picture iteration documentation pass)

- Area affected: Iteration 1-4 planning visibility before MVP kickoff
- Change summary:
  - Added cross-iteration handoff matrix, artifact gate checklist, and evidence pack format to iteration index.
  - Added explicit `Entry Criteria` and `Exit Evidence` sections to Iteration 1-4 boards.
  - Added open clarifications for remaining execution questions (realtime contract artifact scope, KPI/SLO test profile, migration conflict precedence).
  - Added risk-to-task mitigation matrix and updated dependency status for navigation mapping.
  - Linked iteration gate sentences directly to clarification IDs and aligned template parity with active boards.
- Rationale:
  - Provide a full-picture execution plan before coding starts.
  - Make remaining unknowns explicit and trackable instead of implicit assumptions.
- Linked docs updated:
  - `docs/planning/iterations/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (post-hardening precision parity)

- Area affected: Iteration 2-4 execution precision and planning consistency
- Change summary:
  - Added touchpoint/validation gate sections to Iteration 2, 3, and 4 boards for schema parity with Iteration 1.
  - Extended PRD-to-task trace matrix with KPI and discovery-policy coverage rows.
  - Updated template to include touchpoint/validation gate section by default.
  - Normalized stale metadata in `docs/reference/README.md` and dependency status for OpenAPI artifact gate.
- Rationale:
  - Remove remaining non-blocking precision gaps before full parallel MVP execution.
  - Ensure future sprint boards retain deterministic execution quality.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/sprint-board-template.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/reference/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (execution hardening pass)

- Area affected: MVP execution readiness and sprint precision
- Change summary:
  - Expanded dependency register with contract, crypto, navigation-mapping, and voice test-environment gates.
  - Reconciled E2EE risk language with locked MVP requirement for 1:1 and group DM E2EE.
  - Added Iteration 1 OpenAPI artifact gate and touchpoint/validation matrix for all Iteration 1 tasks.
  - Hardened Iteration 2 with group-DM E2EE tasks and navigation-spec trace matrix.
  - Tightened ownership and binary acceptance criteria in Iterations 2-4.
  - Normalized `last_updated` metadata in Iterations 2-4 to 2026-03-04.
- Rationale:
  - Remove remaining contradictions and ambiguity before coding kickoff.
  - Improve deterministic execution quality for AI agents across API/Core/Web/Realtime work.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/sprint-board-template.md`
  - `docs/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (server navigation interaction model lock)

- Area affected: MVP navigation interaction model
- Change summary:
  - Locked dual server navigation mode: sidebar list/folders plus topbar browser-like tabs.
  - Locked saved tabs and tab-folder organization as required navigation capabilities.
  - Locked burger behavior for collapsing/hiding server navigation while inside a server workspace.
  - Updated plan, PRD, navigation spec, and clarifications to align on this model.
- Rationale:
  - Improve navigation speed and organization for large server sets while preserving Discord-like familiarity.
  - Allow focused in-server interaction by temporarily hiding navigation chrome.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/07-ui-navigation-spec.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (navigation design direction lock)

- Area affected: MVP product design and navigation architecture
- Change summary:
  - Locked UI direction to be heavily Discord-inspired with explicit server-navigation deviation.
  - Added canonical navigation/layout specification document for MVP.
  - Locked global `Servers` and `Contacts` hub pages as first-class surfaces.
  - Updated plan, PRD, clarifications, and Iteration 1 board to reference the new navigation authority.
- Rationale:
  - Capture product-level design decisions in canonical docs before implementation expands.
  - Improve navigation scalability for users in large server sets.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/07-ui-navigation-spec.md`
  - `docs/product/README.md`
  - `docs/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (live clarification resolution)

- Area affected: Remaining MVP planning questions and execution precision
- Change summary:
  - Converted live user answers into locked decisions for group DM E2EE, discovery abuse controls, recovery policy, and UI behavior authority.
  - Updated MVP plan and PRD to require group DM E2EE in MVP.
  - Added discovery rate-limit and denylist baseline for MVP discovery.
  - Added mandatory recovery-phrase onboarding policy.
  - Added per-flow UI state tables in Iteration 1 sprint board as the execution authority.
  - Removed file-based quiz workflow and kept clarifications in `docs/product/03-clarifications.md`.
- Rationale:
  - Remove remaining ambiguity that blocked deterministic AI execution on E2 and onboarding paths.
  - Keep decision capture in canonical docs rather than temporary questionnaires.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/03-clarifications.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`
  - `docs/product/README.md`

### 2026-03-04 (readiness detail pass)

- Area affected: MVP execution readiness for Iteration 1 identity/auth/invite work
- Change summary:
  - Locked invite semantics to mode + expiration + max-uses with join-eligibility-only scope.
  - Added MVP Crypto Profile v1 for identity/auth and baseline DM cryptography.
  - Added Iteration 1 OpenAPI endpoint and error-code baseline for identity/invite/auth.
  - Tightened Iteration 1 sprint acceptance criteria for invite exhaustion and nonce replay behavior.
  - Captured remaining product and UX questions for live user-driven resolution.
- Rationale:
  - Remove blocker ambiguity for E2 implementation tasks while preserving unresolved product decisions in a controlled queue.
  - Improve deterministic execution quality for AI agents.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/README.md`
  - `docs/README.md`

### 2026-03-04

- Area affected: Documentation governance and onboarding
- Change summary:
  - Added explicit planning-only onboarding guidance in `README.md`.
  - Added contributor workflow guide at `docs/operations/contributor-guide.md`.
  - Established canonical ADR with `docs/architecture/adr-0001-stack-baseline.md`.
  - Reduced duplicated locked-decision and risk content in `docs/product/02-prd-v1.md` by pointing to canonical sources.
  - Added clarifications and dependency/risk source docs under `docs/product/`.
- Rationale:
  - Improve new-contributor orientation before implementation scaffold exists.
  - Reduce drift risk across PRD and planning docs.
  - Start explicit architecture decision tracking.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/product/02-prd-v1.md`
  - `docs/architecture/README.md`
  - `docs/reference/glossary.md`

### 2026-03-04 (standardization pass)

- Area affected: Documentation standards and canonical ownership boundaries
- Change summary:
  - Removed duplicated task authority from `docs/product/01-mvp-plan.md` and delegated task-level ownership to iteration boards.
  - Removed KPI threshold duplication from `docs/product/01-mvp-plan.md` and kept KPI authority in `docs/product/02-prd-v1.md`.
  - Normalized repeated iteration links to point at `docs/planning/iterations/README.md` from top-level indexes.
  - Added `Quick Context` sections to canonical operational docs to make edit intent explicit.
  - Normalized ADR metadata with `Status: canonical` and explicit `Decision status: accepted`.
  - Added deterministic docs QA checks to contributor workflow.
- Rationale:
  - Eliminate planning drift risk between strategy and sprint docs.
  - Reduce maintenance overhead for link updates.
  - Tighten documentation governance consistency.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd-v1.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/README.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/sprint-board-template.md`
  - `docs/operations/README.md`
  - `docs/operations/contributor-guide.md`
  - `docs/architecture/adr-0001-stack-baseline.md`

## Related Documents

- `docs/planning/iterations/01-sprint-board.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/planning/iterations/03-sprint-board.md`
- `docs/planning/iterations/04-sprint-board.md`
