# Contracts Index

## Document Metadata

- Doc ID: contracts-index
- Owner: API and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-19
- Source of truth: `docs/contracts/README.md`

## Quick Context

- Primary routing index for contract authority and runtime-vs-target-state separation.
- Update this file when contract authority or contract artifact scope changes.
- Latest meaningful change: 2026-05-19 documented REST extractor rejection `ApiError` envelope coverage.

## Purpose

- Separate **current runtime contracts** from **target-state model contracts** to avoid implementation drift confusion.

## Current Runtime Contracts

- REST runtime baseline: `docs/contracts/runtime-rest.openapi.yaml`
- Runtime REST node-administration bootstrap uses `GET /node/connection` for public endpoint/auth metadata and authenticated `GET /node/capabilities` for per-identity owner/admin capability reporting.
- Runtime REST DM schemas describe server-node P2P E2EE envelope fanout, durable explicit static-peer destination node forwarding, catch-up, and internal ack persistence only; recipient-device pairing, LAN/WAN connectivity, endpoint-card, preflight, and parallel-dial routes are intentionally absent.
- Runtime REST dispatch paths that cross the shared `NodeClientTransport` boundary emit stable snake-case provenance mode/profile/reason-code telemetry without changing REST payload schemas.
- Runtime auth transport: HttpOnly `hexrelay_session` cookie or `Authorization: Bearer` token; `x-csrf-token` double-submit is enforced only for cookie-authenticated mutation endpoints.
- Spec-required OpenAPI/AsyncAPI `info.version` fields use `unversioned` and must not be treated as compatibility or rollout signals.
- Some runtime endpoints remain intentionally provisional while tracked in `docs/operations/readiness-corrections-log.md`; call signaling now supports authenticated recipient-targeted live delivery to active websocket sessions for accepted contacts, while DM ciphertext envelopes support recipient-device dispatch and ack over realtime.
- Realtime runtime baseline: `docs/contracts/realtime-events-runtime.asyncapi.yaml`
- Crypto profile baseline: `docs/contracts/crypto-profile.md`

## Contract Parity Status

- Contract parity is strong for the current runtime route set, but it is not a full semantic/runtime-proof gate yet.
- The current route set is covered across high-signal request/response inventory, selected REST semantic checks, shared REST `ApiError` response-schema and schema-shape checks, REST extractor rejection `ApiError` envelope regression coverage, public-route auth absence checks, CSRF header absence plus component name/location, conditional-requiredness, and schema-type checks, internal-token request-header absence, requiredness, and schema-type checks, routed REST path-parameter requiredness, schema-type, and selected format checks, routed REST request-body presence and requiredness checks for `Json<...>` extractors including inline and component-referenced bodies, routed REST request-body JSON media-type exclusivity checks, routed REST request-body absence checks for handlers without request-body extractors, tracked REST DTO required-field including selected `serde(default)` request-field optionality, nullable-field, field-type, selected date-time format, scalar-bound including DM mark-read read-position/unread bounds, enum-domain including DM privacy-policy request/response policy-mode domains, string-pattern, selected array item-pattern, nested array item-schema, and referenced item-field checks, tracked REST response-header schema-type checks, response-builder success-schema checks for local `Json(body).into_response()` handlers, selected receive-side realtime envelope semantics, selected send-side signaling auth/targeting semantics, current send-side signaling success-envelope semantics, shared realtime error-envelope semantics, route-scoped error examples, query requiredness/type/enum/bounds/behavior-tag semantics plus selected string-pattern checks, success-content documentation, and validator regression fixtures.
- Remaining readiness limitations still include broader request/response/auth-behavior semantics beyond the currently tracked REST/realtime slices; treat open `watch` entries in `docs/operations/readiness-corrections-log.md` as the source of truth for those gaps.
- Resume parity work only when:
  - the open CI semantic-depth watch is being hardened,
  - new runtime routes or DTO families land,
  - new stable path, query, DTO type, request-header, response-header, or success semantics become worth enforcing mechanically,
  - validator logic gains a genuinely new branch that needs regression coverage.

## Target-State Model Contracts

- Future REST coverage model: `docs/contracts/mvp-rest.openapi.yaml`
- Future realtime event model: `docs/contracts/realtime-events.asyncapi.yaml`
- Target-state realtime DM events distinguish durable envelope creation, per-device ciphertext dispatch, recipient-device acknowledgement, ack-derived delivery-state updates, and explicit `dm.message.read` receipts; current runtime support remains in `docs/contracts/realtime-events-runtime.asyncapi.yaml`.

## Usage Rule

- If code must match production/runtime behavior now, use current runtime contracts.
- If planning upcoming epics, use target-state model contracts.
