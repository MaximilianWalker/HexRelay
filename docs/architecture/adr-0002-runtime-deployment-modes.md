# ADR-0002: Runtime Deployment Modes

## Document Metadata

- Doc ID: adr-0002-runtime-deployment-modes
- Owner: Architecture maintainers
- Status: accepted
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/architecture/adr-0002-runtime-deployment-modes.md`

## Quick Context

- Primary decision authority for runtime packaging and deployment mode expectations.
- Update this ADR when runtime mode assumptions, packaging boundaries, or deployment topology changes.
- Latest meaningful change: 2026-05-20 aligned deployment modes with the server-node authority decision: one user-facing server maps to one separately runnable node/runtime authority.

## Status

Accepted

## Context

HexRelay is not intended to be only a centrally hosted web application. The primary goal is a downloadable app that users can run off-grid, while still allowing operators to run dedicated server deployments.

Without an explicit runtime decision, product/docs/code discussions drift between browser-hosted assumptions and local-first desktop expectations.

## Decision

- Primary distribution mode is a bundled desktop app.
- Tauri is the default desktop shell unless a later explicit architecture decision replaces it.
- Windows and Linux are mandatory first-class desktop release targets.
- Desktop mode includes UI plus local API/realtime runtime components for user-local operation.
- A desktop app may supervise multiple local server runtimes, but each user-facing server remains a separate node authority with distinct node identity, configuration, and state.
- Local desktop installs may launch UI in either embedded desktop WebView or the user's local browser against localhost.
- Dedicated server mode is also supported for operators who want headless service hosting.
- Dedicated server mode is packaged as a separate service/package family, not as a separate desktop app and not as a default part of the desktop installer.
- Dedicated server runtime remains headless. Authorized node owners/admins manage local or remote dedicated servers through the normal HexRelay app surface, not through a separate dedicated-server UI.
- Dedicated servers may expose authenticated operator/admin APIs for the app to consume. Those APIs are internal management surfaces protected by server authz and operator ingress policy, not public unauthenticated pages.
- Runtime remains multi-component (UI, API service, realtime service) even when desktop packaging installs and supervises local runtime components.
- A single API runtime is not the canonical authority for many unrelated user-facing servers; multi-server app views aggregate distinct node endpoints.
- Browser-only usage remains a compatibility path, not the primary product runtime target.

## Consequences

- Service boundaries stay explicit (API and realtime are server logic, not client bundle logic).
- Desktop packaging must supervise local service lifecycle and local endpoint configuration.
- Desktop multi-server convenience requires node supervision/connection management rather than inserting many independent server rows into one app-owned database.
- Release planning must keep desktop and dedicated-server artifacts separate while allowing shared Rust service code where practical.
- Desktop installer design must avoid silently enabling public/network-facing server behavior for normal users.
- Admin/operator UI work should reuse the app shell and connect to local or remote node endpoints after permission checks instead of creating a second server-specific frontend by default.
- Admin/operator APIs must be scoped and authenticated explicitly; access must not rely on LAN proximity, local network placement, or server discoverability.
- Security boundaries continue to be enforced server-side in API/realtime regardless of where services are hosted.
- CI and smoke tests must validate cross-service behavior, not only isolated unit behavior.

## Alternatives Considered

- Single hosted web app for all users: rejected because it conflicts with local ownership/off-grid target.
- Monolithic single-process app only: rejected because it weakens dedicated server flexibility and operational scaling path.

## Related Documents

- `README.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/operations/03-release-packaging.md`
- `docs/operations/02-dedicated-server-deployment.md`
- `apps/web/README.md`
- `services/api-rs/README.md`
- `services/realtime-rs/README.md`
- `docs/architecture/adr-0004-server-node-authority.md`
