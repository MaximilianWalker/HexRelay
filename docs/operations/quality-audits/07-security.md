# Security Quality Audit

## Metadata

- topic_id: 07-security
- topic: Security
- last_audited: 2026-05-13T06:34:43Z
- source_of_truth: `docs/operations/quality-audits/07-security.md`

## Investigation Focus

- Inspect authn/authz boundaries, input validation, secret handling, dependency exposure, logging, and secure defaults.
- Treat server-readable DM plaintext, private-key upload, or node-bypassing DM transport as severe findings.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-07-20260513-production-origin-scheme-validation | P2 | found | Production origin allowlists can accept non-loopback HTTP browser origins. | `docs/architecture/01-system-overview.md:77` says TLS terminates at ingress and `docs/architecture/01-system-overview.md:97` says dedicated deployments must provide TLS termination; `docs/operations/02-dedicated-server-deployment.md:74` says admin/operator traffic is protected by ingress, TLS, origin, session, and CSRF. API config reads `API_ALLOWED_ORIGINS` at `services/api-rs/src/config.rs:107`, only rejects empty lists at `services/api-rs/src/config.rs:215`, then production checks cover dev-testing, DB, cookies, signing keys, and internal tokens at `services/api-rs/src/config.rs:288`, `:295`, `:309`, `:327`, and `:334` without rejecting non-loopback `http://` origins; the router applies those origins to credentialed CORS at `services/api-rs/src/app/router.rs:50` and `:57`. Realtime config similarly reads `REALTIME_ALLOWED_ORIGINS` at `services/realtime-rs/src/config.rs:58`, only rejects empty lists at `services/realtime-rs/src/config.rs:102`, production-checks tokens/dev faults at `services/realtime-rs/src/config.rs:195`, `:204`, and `:211`, and then compares websocket `Origin` directly against configured strings at `services/realtime-rs/src/transport/ws/handlers/gateway.rs:771` and `:782`. | Add shared origin parsing/validation that rejects non-loopback `http://` origins in production for both API and realtime, while preserving loopback development origins; add config tests for rejected production HTTP origins and accepted HTTPS origins. | 2026-05-13T06:34:43Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-07-20260513-web-csp-connect-src-broad | P2 | fixed | The web CSP allowed arbitrary network egress from browser-executed code. | `apps/web/next.config.ts` now builds `connect-src` from `NEXT_PUBLIC_API_BASE_URL`, `NEXT_PUBLIC_REALTIME_WS_URL`, and `'self'`, keeps wildcard loopback sources development-only, rejects non-loopback plaintext endpoints outside development, and `apps/web/next.config.test.ts` verifies production omits broad `http:`, `https:`, `ws:`, and `wss:` sources while development keeps only loopback allowances. | 2026-05-19T06:30:41Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T06:34:43Z | Codex | Added 2 P2 found findings about production origin scheme validation and broad web CSP network egress. |
