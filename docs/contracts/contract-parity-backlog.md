# Contract Parity Closeout

## Document Metadata

- Doc ID: contract-parity-closeout
- Owner: API and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-06
- Source of truth: `docs/contracts/contract-parity-backlog.md`

## Quick Context

- Final status note for the runtime-vs-contract parity program.
- Use this file to understand what is closed now and what should trigger future parity work.
- Latest meaningful change: 2026-04-06 finalized the closeout status after request/response schema, route-scoped example, query semantics, success-content, and regression-fixture breadth were all completed for the current route set.

## Purpose

- Record the completed contract-parity coverage baseline for the current route set.
- Make future parity additions intentional instead of reviving stale backlog slices.
- Preserve the trigger conditions for when parity work should start again.

## Coverage Snapshot

- Covered well: route inventory, realtime inventory, global error-code inventory, exact session-auth security-scheme parity, internal-auth/header/security parity for the internal presence watcher route, CSRF parameter parity, request-body presence, request/response schema-ref parity including request/response alias normalization and direct mismatch regressions, success-status presence, selected error-status presence including extractor-backed `403`/`404` paths and helper/delegate `400`/`500` flows, path/query parameter presence, response-header parity, auth cookie semantics, route-scoped `ApiError.code` parity for high-signal routes, broad route-scoped error-example parity including status-specific server-channel mutation checks, deterministic regression fixtures for missing auth/status/schema/content/header branches, tracked query semantics for the current safe mechanically asserted rule set, and success-content parity across the current meaningful route families.
- Residual only: opportunistic future breadth for newly added routes, newly introduced stable semantics, or new validator branches; no current high-signal parity gap is known.

## Current State

- No required parity breadth slices remain open for the current route set.
- New parity work should be triggered only by:
  - newly added runtime routes or DTO families,
  - newly stabilized query or success semantics worth enforcing,
  - newly added validator logic that needs regression fixtures.

## Closed Categories

- Request-schema breadth: complete for current routed JSON request bodies.
- Response-schema breadth: complete for current routed JSON success bodies.
- Internal header and auth cookie/header parity: complete for current runtime surfaces.
- Route-scoped `ApiError.code` and error-example breadth: complete for the current high-signal route families.
- Query semantics: complete for the current safe mechanically asserted rule set.
- Success-content parity: complete for the current meaningful route families.
- Regression fixture coverage: complete for the current validator branches.

## Future Trigger Conditions

- Add parity work when a new runtime route or DTO family lands.
- Add parity work when a stable query or success semantic becomes worth enforcing mechanically.
- Add regression fixtures when validator logic gains a genuinely new branch.
