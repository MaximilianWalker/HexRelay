# ADR-0002: Runtime Deployment Modes

## Status

Accepted

## Context

HexRelay is not intended to be only a centrally hosted web application. The primary goal is a downloadable app that users can run off-grid, while still allowing operators to run dedicated server deployments.

Without an explicit runtime decision, product/docs/code discussions drift between browser-hosted assumptions and local-first desktop expectations.

## Decision

- Primary distribution mode is a bundled desktop app.
- Desktop mode includes UI plus local API/realtime runtime components.
- Local desktop installs may launch UI in either embedded desktop WebView or the user's local browser against localhost.
- Dedicated server mode is also supported for operators who want headless service hosting.
- Runtime remains multi-component (UI, API service, realtime service) even when distributed as one installer.
- Browser-only usage remains a compatibility path, not the primary product runtime target.

## Consequences

- Service boundaries stay explicit (API and realtime are server logic, not client bundle logic).
- Desktop packaging must supervise local service lifecycle and local endpoint configuration.
- Security boundaries continue to be enforced server-side in API/realtime regardless of where services are hosted.
- CI and smoke tests must validate cross-service behavior, not only isolated unit behavior.

## Alternatives Considered

- Single hosted web app for all users: rejected because it conflicts with local ownership/off-grid target.
- Monolithic single-process app only: rejected because it weakens dedicated server flexibility and operational scaling path.

## Related Documents

- `README.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `apps/web/README.md`
- `services/api-rs/README.md`
- `services/realtime-rs/README.md`
