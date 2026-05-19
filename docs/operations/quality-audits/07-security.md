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
| QA-07-20260513-web-csp-connect-src-broad | P2 | found | The web CSP allows arbitrary network egress from browser-executed code. | `apps/web/next.config.ts:16` builds response security headers, but `apps/web/next.config.ts:26` sets `connect-src 'self' http: https: ws: wss:`, allowing connections to any HTTP(S) or websocket endpoint. The app already has explicit runtime endpoint config in `apps/web/lib/env.ts:4`, `:5`, `:10`, and `:12`, so the CSP does not use the known API/realtime origins to contain exfiltration if script execution is compromised. | Derive production `connect-src` from `NEXT_PUBLIC_API_BASE_URL`, `NEXT_PUBLIC_REALTIME_WS_URL`, and `self` with loopback allowances only for development, then add a header/regression check that arbitrary external origins are not present. | 2026-05-13T06:34:43Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-07-20260513-production-origin-scheme-validation | P2 | fixed | Production origin allowlists can accept non-loopback HTTP browser origins. | Added shared browser-origin parsing in `crates/communication-core/src/config/mod.rs`, wired `API_ALLOWED_ORIGINS` and `REALTIME_ALLOWED_ORIGINS` through it, and rejected production non-loopback `http://` origins while preserving HTTPS and loopback HTTP origins. Focused API and realtime config regressions failed before the fix and passed after it with `cargo test -p api-rs allowed_origin --all-features` and `cargo test -p realtime-rs allowed_origin --all-features`; shared parser coverage passed with `cargo test -p communication-core config --all-features`. | 2026-05-19 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-19 | Codex issue remediator | Fixed `QA-07-20260513-production-origin-scheme-validation` with shared production browser-origin scheme validation for API and realtime config. |
| 2026-05-13T06:34:43Z | Codex | Added 2 P2 found findings about production origin scheme validation and broad web CSP network egress. |
