# Contracts Index

## Document Metadata

- Doc ID: contracts-index
- Owner: API and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/contracts/README.md`

## Quick Context

- Primary routing index for contract authority and runtime-vs-target-state separation.
- Update this file when contract authority or contract artifact scope changes.
- Latest meaningful change: 2026-05-11 aligned runtime DM fanout contract with durable outbound static-peer forwarding state.

## Purpose

- Separate **current runtime contracts** from **target-state model contracts** to avoid implementation drift confusion.

## Current Runtime Contracts

- REST runtime baseline: `docs/contracts/runtime-rest-v1.openapi.yaml`
- REST legacy alias path (non-authoritative): `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
- Runtime REST DM schemas describe server-node P2P E2EE envelope fanout, durable explicit static-peer destination node forwarding, catch-up, and internal ack persistence only; recipient-device pairing, LAN/WAN connectivity, endpoint-card, preflight, and parallel-dial routes are intentionally absent.
- Runtime auth transport: HttpOnly `hexrelay_session` cookie or `Authorization: Bearer` token; `x-csrf-token` double-submit is enforced only for cookie-authenticated mutation endpoints.
- Some runtime endpoints remain intentionally provisional while tracked in `docs/operations/readiness-corrections-log.md`; call signaling remains self-targeted loopback only, while DM ciphertext envelopes now support recipient-device dispatch and ack over realtime.
- Realtime runtime baseline: `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`
- Crypto profile baseline: `docs/contracts/crypto-profile-v1.md`

## Contract Parity Status

- Contract parity is strong for the current runtime route set, but it is not a full semantic/runtime-proof gate yet.
- The current route set is covered across high-signal request/response inventory, selected REST semantic checks, shared REST `ApiError` response-schema checks, tracked REST DTO required-field checks, selected receive-side realtime envelope semantics, selected send-side signaling auth/targeting semantics, current send-side signaling success-envelope semantics, shared realtime error-envelope semantics, route-scoped error examples, query semantics, success-content documentation, and validator regression fixtures.
- Remaining readiness limitations still include broader request/response/auth-behavior semantics beyond the currently tracked REST/realtime slices; treat open `watch` entries in `docs/operations/readiness-corrections-log.md` as the source of truth for those gaps.
- Resume parity work only when:
  - the open CI semantic-depth watch is being hardened,
  - new runtime routes or DTO families land,
  - new stable query or success semantics become worth enforcing mechanically,
  - validator logic gains a genuinely new branch that needs regression coverage.

## Target-State Model Contracts

- Future REST coverage model: `docs/contracts/mvp-rest-v1.openapi.yaml`
- Future realtime event model: `docs/contracts/realtime-events-v1.asyncapi.yaml`
- Target-state realtime DM events distinguish durable envelope creation, per-device ciphertext dispatch, recipient-device acknowledgement, ack-derived delivery-state updates, and explicit `dm.message.read` receipts; current runtime support remains in `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`.

## Usage Rule

- If code must match production/runtime behavior now, use current runtime contracts.
- If planning upcoming epics, use target-state model contracts.
