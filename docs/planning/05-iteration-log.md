# HexRelay Iteration Log

## Document Metadata

- Doc ID: iteration-log
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/planning/05-iteration-log.md`

## Quick Context

- Primary edit location for project-level delivery changes across iterations.
- Do not duplicate sprint task detail here; link to iteration boards when needed.
- Latest meaningful change: 2026-03-04 execution hardening aligned E2EE scope, dependencies, and sprint task precision.

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
