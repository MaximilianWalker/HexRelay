# Contract Parity Backlog

## Document Metadata

- Doc ID: contract-parity-backlog
- Owner: API and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-06
- Source of truth: `docs/contracts/contract-parity-backlog.md`

## Quick Context

- Primary working backlog for remaining runtime-vs-contract parity hardening.
- Use this file to decide the next meaningful `scripts/validate-contract-parity.sh` slices.
- Latest meaningful change: 2026-04-06 closed the last real request-schema breadth exception by aligning the friend-request create DTO name with the contract and removing the stale validator route special-case.

## Purpose

- Track the major parity categories still not enforced by CI.
- Keep the next slices grouped into medium-sized, coherent validator changes.
- Avoid losing context after the recent run of small response-path parity passes.

## Current Coverage Snapshot

- Covered well: route inventory, realtime inventory, global error-code inventory, exact session-auth security-scheme parity, internal-auth/header/security parity for the internal presence watcher route, CSRF parameter parity, request-body presence, request/response schema-ref parity including request/response alias normalization and direct mismatch regressions, success-status presence, selected error-status presence including extractor-backed `403`/`404` paths and helper/delegate `400`/`500` flows, path/query parameter presence, response-header parity, auth cookie semantics, route-scoped `ApiError.code` parity for high-signal routes, broad route-scoped error-example parity including status-specific server-channel mutation checks, deterministic regression fixtures for missing auth/status/schema/content/header branches, tracked query semantics for the current safe mechanically asserted rule set, and success-content parity across the current meaningful route families.
- Residual only: opportunistic future breadth for newly added routes, newly introduced stable semantics, or new validator branches; no current high-signal parity gap is known.

## Prioritized Todo List

1. Close out request schema parity breadth
- Verify each routed `Json<T>` request body maps to the correct OpenAPI schema ref, not just that `requestBody` exists.
- Start with high-churn surfaces in `services/api-rs/src/transport/http/handlers/dm.rs`, `services/api-rs/src/transport/http/handlers/friends.rs`, and `services/api-rs/src/transport/http/handlers/block_mute.rs`.
- Status: completed for the currently routed JSON request-body surfaces; the last real breadth exception (`FriendRequestCreate` vs `FriendRequestCreateRequest`) is now aligned and the stale validator route key for identity registration is corrected.

2. Close out response schema parity breadth
- Verify routed `200`/`201` JSON responses point at the correct OpenAPI schema ref for the returned DTO.
- Start with `services/api-rs/src/transport/http/handlers/auth.rs`, `services/api-rs/src/transport/http/handlers/presence.rs`, `services/api-rs/src/transport/http/handlers/server_channels.rs`, and `services/api-rs/src/transport/http/handlers/dm.rs`.
- Status: completed for the current routed JSON success-body surfaces; follow-up is only needed if future nested/schema-shape drift or a new routed alias family appears.

3. Add internal header parity
- Enforce non-CSRF request-header documentation for runtime-required headers.
- First target: `x-hexrelay-internal-token` on `/v1/internal/presence/watchers/{identity_id}` from `services/api-rs/src/transport/http/handlers/presence.rs`.
- Status: completed for the currently known internal-token route surface.

4. Add auth response-header and cookie parity
- Enforce documented `Set-Cookie` and cookie-clearing behavior for auth session routes.
- First target: `/v1/auth/verify` and `/v1/auth/sessions/revoke` in `services/api-rs/src/transport/http/handlers/auth.rs`.
- Status: completed for auth verify/revoke cookie issue-vs-clear semantics on `hexrelay_session` and `hexrelay_csrf`.

5. Add route-scoped `ApiError.code` parity
- Check the concrete error codes each route can emit instead of validating only the global `ApiError.code` enum inventory.
- First target routes: friend-request transitions in `services/api-rs/src/transport/http/handlers/friends.rs` and message mutation routes in `services/api-rs/src/transport/http/handlers/server_channels.rs`.
- Status: effectively completed for the current high-signal routed families; future expansion is optional and only worthwhile when new routes add distinct error-code behavior.

6. Close out route-scoped error example breadth
- Validate that documented route-level `401`/`403`/`404` examples and descriptions match runtime failure meaning, not just status presence.
- First cleanup target: `/v1/friends/requests/{request_id}/accept` in `docs/contracts/runtime-rest-v1.openapi.yaml`.
- Status: effectively completed for the current high-signal routed families, including auth register/challenge/verify/revoke routes, friend-request list/create/mutation/bootstrap routes, block/mute create routes, invite lifecycle routes, discovery validation routes, server membership read routes, DM thread/history read routes, DM control-plane bad-request examples, internal-auth `401` examples, and server-channel mutation status-specific examples.

7. Expand query semantics beyond the tracked rule table
- Cover more query/filter semantics where runtime behavior is stable enough to assert mechanically.
- First targets: `services/api-rs/src/transport/http/handlers/directory.rs` and remaining safe rules in `services/api-rs/src/transport/http/handlers/discovery.rs`.
- Status: effectively completed for the current safe mechanically asserted rule set; `ServerListQuery` and `ContactListQuery` cover blank-search normalization plus case-insensitive matching, and `DiscoveryUserListQuery` covers default, trim-before-enum, blank-query normalization, case-insensitive matching, and limit clamp semantics. Future additions should only land when new stable query rules are worth enforcing.

8. Close out success content parity breadth
- Enforce that JSON success routes document response content and true no-content routes stay `204` without body docs.
- First target files: `services/api-rs/src/transport/http/handlers/health.rs`, `services/api-rs/src/transport/http/handlers/friends.rs`, and `services/api-rs/src/transport/http/handlers/block_mute.rs`.
- Status: effectively completed for the current meaningful route families; auth/invite, DM fanout/profile-device control, server-channel lifecycle, social-graph handshake, DM connectivity setup/control, and the remaining simple block/list/read routes now have branch-specific success descriptions/examples.

9. Separate internal-auth parity from session-auth parity
- Add a dedicated validator path for internal-token-protected endpoints rather than treating them as a one-off documented header.
- Start with `services/api-rs/src/transport/http/handlers/presence.rs` and the matching OpenAPI route block.
- Status: completed for the current internal presence watcher route family, including dedicated header/security handling and required `internal_token_invalid` example coverage.

10. Close out validator regression fixture coverage
- Add a small deterministic test layer around the validator so future parity expansions do not regress silently.
- Cover tricky handlers first: `auth.rs`, `friends.rs`, `dm.rs`, `server_channels.rs`, and `presence.rs`.
- Status: effectively completed for the current validator; fixtures now cover missing route examples, auth cookie semantics, discovery query semantics, DM control-plane examples, invite create examples, DM fanout validation, internal-auth header/security/example gaps, session-auth security/status gaps, missing 401/500 branches including non-auth helper/delegate `500` flows, request/response schema alias handling, direct request-schema mismatch, response-header parity, requestBody/CSRF gaps, no-content success-schema regressions, and server-channel status-specific example checks with passing/failing counterparts. Add more only if a genuinely new validator branch with regression value appears.

## Current State

- No required parity breadth slices remain open for the current route set.
- New parity work should be triggered only by:
  - newly added runtime routes or DTO families,
  - newly stabilized query or success semantics worth enforcing,
  - newly added validator logic that needs regression fixtures.

## Working Rule

- Prefer grouped slices that close one coherent parity class across multiple routes.
- Avoid returning to one-status-at-a-time PRs unless a review finding forces a tiny follow-up.
