# ADR-0002: Runtime Deployment Modes

## Document Metadata

- Doc ID: adr-0002-runtime-deployment-modes
- Owner: Architecture maintainers
- Status: accepted
- Scope: repository
- last_updated: 2026-05-07
- Source of truth: `docs/architecture/adr-0002-runtime-deployment-modes.md`

## Quick Context

- Primary decision authority for runtime packaging and deployment mode expectations.
- Update this ADR when runtime mode assumptions, packaging boundaries, or deployment topology changes.
- Latest meaningful change: 2026-05-07 clarified Windows/Linux release parity, Tauri as the default desktop shell, and the dedicated-server package boundary.

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
- Local desktop installs may launch UI in either embedded desktop WebView or the user's local browser against localhost.
- Dedicated server mode is also supported for operators who want headless service hosting.
- Dedicated server mode is packaged as a separate service/package family, not as a separate desktop app and not as a default part of the desktop installer.
- Runtime remains multi-component (UI, API service, realtime service) even when desktop packaging installs and supervises local runtime components.
- Browser-only usage remains a compatibility path, not the primary product runtime target.

## Consequences

- Service boundaries stay explicit (API and realtime are server logic, not client bundle logic).
- Desktop packaging must supervise local service lifecycle and local endpoint configuration.
- Release planning must keep desktop and dedicated-server artifacts separate while allowing shared Rust service code where practical.
- Desktop installer design must avoid silently enabling public/network-facing server behavior for normal users.
- Security boundaries continue to be enforced server-side in API/realtime regardless of where services are hosted.
- CI and smoke tests must validate cross-service behavior, not only isolated unit behavior.

## Alternatives Considered

- Single hosted web app for all users: rejected because it conflicts with local ownership/off-grid target.
- Monolithic single-process app only: rejected because it weakens dedicated server flexibility and operational scaling path.

## Related Documents

- `README.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/operations/03-release-packaging.md`
- `docs/operations/02-dedicated-server-deployment.md`
- `apps/web/README.md`
- `services/api-rs/README.md`
- `services/realtime-rs/README.md`
