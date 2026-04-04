# Contracts Index

## Document Metadata

- Doc ID: contracts-index
- Owner: API and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-16
- Source of truth: `docs/contracts/README.md`

## Quick Context

- Primary routing index for contract authority and runtime-vs-target-state separation.
- Update this file when contract authority or contract artifact scope changes.
- Latest meaningful change: 2026-04-04 started grouped schema and header parity hardening for request/response body refs, internal auth headers, and auth cookie response headers.

## Purpose

- Separate **current runtime contracts** from **target-state model contracts** to avoid implementation drift confusion.

## Current Runtime Contracts

- REST runtime baseline: `docs/contracts/runtime-rest-v1.openapi.yaml`
- REST legacy alias path (non-authoritative): `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
- Runtime auth transport: HttpOnly `hexrelay_session` cookie or `Authorization: Bearer` token; `x-csrf-token` double-submit is enforced only for cookie-authenticated mutation endpoints.
- Some runtime endpoints remain intentionally provisional while tracked in `docs/operations/readiness-corrections-log.md`; realtime signaling remains self-targeted loopback only until recipient fanout exists.
- Realtime runtime baseline: `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml`
- Crypto profile baseline: `docs/contracts/crypto-profile-v1.md`
- Remaining contract-parity backlog: `docs/contracts/contract-parity-backlog.md`

## Target-State Model Contracts

- Future REST coverage model: `docs/contracts/mvp-rest-v1.openapi.yaml`
- Future realtime event model: `docs/contracts/realtime-events-v1.asyncapi.yaml`

## Usage Rule

- If code must match production/runtime behavior now, use current runtime contracts.
- If planning upcoming epics, use target-state model contracts.
