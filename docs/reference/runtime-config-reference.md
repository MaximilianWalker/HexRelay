# HexRelay Runtime Config Reference

## Document Metadata

- Doc ID: runtime-config-reference
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-04-03
- Source of truth: `docs/reference/runtime-config-reference.md`

## Quick Context

- Purpose: provide the canonical runtime environment/config reference for `services/api-rs` and `services/realtime-rs`.
- Primary edit location: update this file whenever `services/*/src/config.rs` or `services/*/.env.example` changes.
- Latest meaningful change: 2026-04-03 created the first canonical service runtime-config reference and aligned onboarding docs to point here.

## Purpose

- Centralize runtime env/config behavior for the Rust services.
- Keep onboarding docs short by linking here instead of duplicating variable inventories.
- Make production-only validation rules visible outside code.

## Scope and Authority

- This document covers runtime env/config for:
  - `services/api-rs`
  - `services/realtime-rs`
- Product/policy defaults remain separately owned by:
  - `docs/product/09-configuration-defaults-register.md`
- Implementation truth lives in:
  - `services/api-rs/src/config.rs`
  - `services/realtime-rs/src/config.rs`
  - `services/api-rs/.env.example`
  - `services/realtime-rs/.env.example`

## Resolution Model

- Process environment overrides code defaults.
- `.env.example` files are bootstrap samples, not the full narrative authority.
- `development` and `production` have different validation strictness.
- Secret-bearing values must stay out of committed machine-specific files.

## `api-rs` Runtime Config

| Variable | Default | Production requirement | Notes |
|---|---|---|---|
| `API_BIND` | `127.0.0.1:8080` | required | host:port bind address |
| `API_ENVIRONMENT` | `development` | required | `development` or `production` |
| `API_ALLOW_PUBLIC_IDENTITY_REGISTRATION` | `false` | optional | keep disabled until trusted claim flow exists |
| `API_NODE_FINGERPRINT` | `hexrelay-local-fingerprint` | must be non-default | deployment identity marker |
| `API_DATABASE_URL` | local dev Postgres URL | must be non-default | durable API state store |
| `API_ALLOWED_ORIGINS` | `http://localhost:3002,http://127.0.0.1:3002` | required | must contain at least one origin |
| `API_TRUST_PROXY_HEADERS` | `false` | optional | enable only behind trusted proxy/header sanitization |
| `API_CHANNEL_DISPATCH_INTERNAL_TOKEN` | dev default token | must be non-default | API -> realtime channel dispatch credential |
| `API_PRESENCE_WATCHER_INTERNAL_TOKEN` | dev default token | must be non-default | realtime -> API presence watcher credential |
| `API_REALTIME_BASE_URL` | `http://127.0.0.1:8081` | required | absolute URL; non-loopback hosts must use `https` |
| `API_PRESENCE_REDIS_URL` | unset | optional | enables Redis-backed presence snapshot source |
| `API_DISCOVERY_DENYLIST` | unset | optional | CSV denylist for discovery filtering |
| `API_SESSION_SIGNING_KEYS` | unset in code, set in example | required in production | preferred keyring format: `key_id:secret,...` |
| `API_SESSION_SIGNING_KEY_ID` | `v1` when unset | required with keyring | active signing key id; when using `API_SESSION_SIGNING_KEYS`, the selected id must exist in the keyring |
| `API_SESSION_SIGNING_KEY` | legacy fallback | avoid in production | local compatibility fallback only |
| `API_SESSION_COOKIE_SECURE` | `false` | must be `true` | required for production cookies |
| `API_SESSION_COOKIE_SAME_SITE` | `Lax` | required | `Strict`, `Lax`, or `None`; `None` requires secure cookie |
| `API_SESSION_COOKIE_DOMAIN` | unset | optional | requires `API_SESSION_COOKIE_SECURE=true` |
| `API_AUTH_CHALLENGE_RATE_LIMIT` | `30` | optional | positive integer |
| `API_AUTH_VERIFY_RATE_LIMIT` | `30` | optional | positive integer |
| `API_DISCOVERY_QUERY_RATE_LIMIT` | `30` | optional | positive integer |
| `API_INVITE_CREATE_RATE_LIMIT` | `20` | optional | positive integer |
| `API_INVITE_REDEEM_RATE_LIMIT` | `40` | optional | positive integer |
| `API_RATE_LIMIT_WINDOW_SECONDS` | `60` | optional | must be greater than zero |
| `RUST_LOG` | unset in code | optional | standard Rust logging filter |

## `realtime-rs` Runtime Config

| Variable | Default | Production requirement | Notes |
|---|---|---|---|
| `REALTIME_BIND` | `127.0.0.1:8081` | required | host:port bind address |
| `REALTIME_ENVIRONMENT` | `development` | required | `development` or `production` |
| `REALTIME_API_BASE_URL` | `http://127.0.0.1:8080` | required | absolute URL; non-loopback hosts must use `https` |
| `REALTIME_REQUIRE_API_HEALTH_ON_START` | `true` | optional | fail startup if API health is unavailable |
| `REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN` | dev default token | must be non-default | authorizes protected internal channel publish ingress |
| `REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN` | dev default token | must be non-default | outbound watcher lookup credential toward API |
| `REALTIME_PRESENCE_REDIS_URL` | unset | optional | enables Redis-backed presence/replay authority |
| `REALTIME_TRUST_PROXY_HEADERS` | `false` | optional | enable only behind trusted proxy/header sanitization |
| `REALTIME_ALLOWED_ORIGINS` | `http://localhost:3002,http://127.0.0.1:3002` | required | websocket/browser origin allowlist |
| `REALTIME_WS_CONNECT_RATE_LIMIT` | `60` | optional | positive integer |
| `REALTIME_RATE_LIMIT_WINDOW_SECONDS` | `60` | optional | positive integer |
| `REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES` | `16384` | optional | integer >= `256` |
| `REALTIME_WS_MESSAGE_RATE_LIMIT` | `120` | optional | positive integer |
| `REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS` | `60` | optional | positive integer |
| `REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY` | `3` | optional | positive integer |
| `REALTIME_WS_AUTH_GRACE_SECONDS` | `0` | optional | enables bounded cache-based auth grace mode |
| `REALTIME_WS_AUTH_CACHE_MAX_ENTRIES` | `10000` | optional | positive integer |
| `RUST_LOG` | unset in code | optional | standard Rust logging filter |

## Local Development Minimum

- Required local bootstrap values:
  - `API_SESSION_SIGNING_KEYS`
  - `API_SESSION_SIGNING_KEY_ID`
- Common local-safe defaults already exist in both `*.env.example` files.
- Use these docs for local startup guidance:
  - `README.md`
  - `docs/operations/dev-prerequisites.md`

## Dedicated Server Minimum

- API secrets/config that must be reviewed explicitly:
  - `API_DATABASE_URL`
  - `API_SESSION_SIGNING_KEYS`
  - `API_SESSION_SIGNING_KEY_ID`
  - `API_CHANNEL_DISPATCH_INTERNAL_TOKEN`
  - `API_PRESENCE_WATCHER_INTERNAL_TOKEN`
- Realtime secrets/config that must be reviewed explicitly:
  - `REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN`
  - `REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN`
- Dedicated deployments should also review:
  - origin allowlists
  - proxy-header trust flags
  - cookie security/domain settings
  - auth grace/cache settings

## Change Rule

- If `services/api-rs/src/config.rs`, `services/realtime-rs/src/config.rs`, `services/api-rs/.env.example`, or `services/realtime-rs/.env.example` changes, update this document and `docs/README.md` in the same PR.

## Related Documents

- `docs/architecture/01-system-overview.md`
- `docs/operations/dev-prerequisites.md`
- `docs/operations/01-mvp-runbook.md`
- `docs/product/09-configuration-defaults-register.md`
