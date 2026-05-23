# HexRelay Runtime Config Reference

## Document Metadata

- Doc ID: runtime-config-reference
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/reference/runtime-config-reference.md`

## Quick Context

- Purpose: provide the canonical runtime environment/config reference for `services/api-rs` and `services/realtime-rs`.
- Primary edit location: update this file whenever `services/*/src/config.rs` or `services/*/.env.example` changes.
- Latest meaningful change: 2026-05-20 aligned server identity config around `server_id`, signed server descriptors, and server-to-server private meshes.

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
| `API_ENABLE_DEV_TESTING` | `false` | must be `false` | enables local-only fixture/session testing endpoints in development |
| `API_SERVER_ID` | `hexrelay-local-server` | must be non-default | deployment identity marker |
| `API_SERVER_OWNER_IDENTITY_IDS` | unset | optional bootstrap config | CSV of identity ids that receive server-owner scope through `/server/capabilities`; owner implies admin |
| `API_SERVER_ADMIN_IDENTITY_IDS` | unset | optional bootstrap config | CSV of identity ids that receive server-admin/operator scope through `/server/capabilities` |
| `API_DATABASE_URL` | local dev Postgres URL | must be non-default | durable API state store |
| `API_ALLOWED_ORIGINS` | `http://localhost:3002,http://127.0.0.1:3002` | required | must contain at least one origin |
| `API_TRUST_PROXY_HEADERS` | `false` | optional | enable only behind trusted proxy/header sanitization |
| `API_CHANNEL_DISPATCH_INTERNAL_TOKEN` | dev default token | must be non-default | API -> realtime channel dispatch credential |
| `API_PRESENCE_WATCHER_INTERNAL_TOKEN` | dev default token | must be non-default | realtime -> API presence watcher credential |
| `API_REALTIME_BASE_URL` | `http://127.0.0.1:8081` | required | absolute URL; non-loopback hosts must use `https` |
| `API_PRESENCE_REDIS_URL` | unset | optional config knob | enables Redis-backed presence snapshot source; required for the reviewed dedicated single-server deployment baseline |
| `API_DISCOVERY_DENYLIST` | unset | optional | CSV denylist for discovery filtering |
| `API_LOCAL_SERVER_DESCRIPTOR_JSON` | unset | optional | signed local server descriptor JSON; required with `API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64` for authenticated server-to-server forwarding and descriptor `server_id` must match `API_SERVER_ID` |
| `API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64` | unset | optional secret | base64-encoded Ed25519 PKCS#8 server-to-server signing key; required with `API_LOCAL_SERVER_DESCRIPTOR_JSON`, must match the descriptor public key, and must stay server-local |
| `API_STATIC_PEER_DESCRIPTORS_JSON` | unset | optional | JSON array of signed server descriptors for static private-mesh peers; each descriptor is signature/TTL/policy validated at startup |
| `API_STATIC_PEER_INVITES_JSON` | unset | optional | JSON array of signed peer-invite envelopes; each envelope contains an issuer descriptor plus signed invite, must validate against `API_SERVER_ID` when subject-bound, and joins the same static peer registry as configured descriptors |
| `API_REVOKED_STATIC_PEER_INVITE_IDS` | unset | optional | CSV of signed peer-invite IDs refused during `API_STATIC_PEER_INVITES_JSON` validation; use to invalidate compromised or superseded invite envelopes without changing descriptor trust |
| `API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS` | `86400` | optional | positive integer TTL ceiling applied to configured static peer descriptors |
| `API_SESSION_SIGNING_KEYS` | unset in code, set in example | required in production | preferred keyring format: `key_id:secret,...` |
| `API_SESSION_SIGNING_KEY_ID` | `primary` when unset | required with keyring | active signing key id; when using `API_SESSION_SIGNING_KEYS`, the selected id must exist in the keyring |
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
| `REALTIME_PRESENCE_REDIS_URL` | unset | optional config knob | enables Redis-backed presence/replay authority; required for the reviewed dedicated single-server deployment baseline |
| `REALTIME_STATIC_PEER_DESCRIPTORS_JSON` | unset | optional | JSON array of signed server descriptors for static private-mesh peers; each descriptor is signature/TTL/policy validated at startup |
| `REALTIME_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS` | `86400` | optional | positive integer TTL ceiling applied to configured static peer descriptors |
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
| `REALTIME_ENABLE_DEV_FAULTS` | `false` | must be `false` | enables internal local-only realtime delay/drop/disconnect fault hooks; non-loopback binds require a non-default channel dispatch token |
| `RUST_LOG` | unset in code | optional | standard Rust logging filter |

## Dev-Only Testing Flags

### `API_ENABLE_DEV_TESTING`

- Default: `false`.
- Production requirement: must remain `false`.
- Purpose: exposes fixture-backed testing profile/session endpoints for local manual and browser testing.
- Safe local use requires `API_ENVIRONMENT=development`, loopback API bind, loopback database host, and local browser origins.
- The endpoint is an adoption aid for `docs/operations/local-runtime-testing-quickstart.md`; it is not an auth bypass for production or shared environments.

### `REALTIME_ENABLE_DEV_FAULTS`

- Default: `false`.
- Production requirement: must remain `false`.
- Purpose: exposes internal local app-fault hooks used by network simulation profiles such as `flaky-mobile`.
- Non-loopback realtime binds require a non-default `REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN` when this flag is enabled.
- Run `npm run network -- --reset` after manual app-fault testing so the realtime process returns to baseline behavior.

## Peer Invite Issuance Tool

- `cargo run -p api-rs --bin issue_peer_invite -- --subject-server-id SERVER_ID` prints a signed `PeerInviteEnvelope` JSON object; add it as an element of the recipient's `API_STATIC_PEER_INVITES_JSON` array.
- The tool reads `API_LOCAL_SERVER_DESCRIPTOR_JSON` and `API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64`, validates that the private key matches the descriptor public key, and enforces `API_SERVER_ID` when it is set.
- Generated invites default to subject-bound, single-use, `private_allowlist` discovery, and one-hour TTL. Unbound bearer invites require explicit `--allow-unbound`.
- `API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS` is reused as the issuance TTL ceiling so generated envelopes match startup validation defaults.

## Local Server Identity Generation Tool

- `cargo run -p api-rs --bin generate_server_identity -- --server-id SERVER_ID --address URL` prints a signed local server descriptor plus base64 Ed25519 PKCS#8 private key for `API_LOCAL_SERVER_DESCRIPTOR_JSON` and `API_LOCAL_SERVER_PRIVATE_KEY_PKCS8_BASE64`.
- Defaults are private-mesh oriented: `private_peers`, `private_allowlist`, `invite_token`, `none` relay, `local_recipients_only` forwarding, `durable_encrypted_envelopes`, and `hexrelay-server-http`.
- The generator validates the descriptor policy shape and signature before printing output. Incoherent policy combinations fail instead of producing unusable config.
- `API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS` is reused as the descriptor TTL ceiling so generated local identity config matches API startup validation.

## Local Development Minimum

- Required local bootstrap values:
  - `API_SESSION_SIGNING_KEYS`
  - `API_SESSION_SIGNING_KEY_ID`
- Common local-safe defaults already exist in both `*.env.example` files.
- Use these docs for local startup guidance:
  - `README.md`
  - `docs/operations/dev-prerequisites.md`

## Dedicated Server Minimum

- Reviewed dedicated single-server baseline services:
  - Postgres
  - Redis
  - `api-rs`
  - `realtime-rs`
  - TLS-capable ingress/reverse proxy
- Feature-scoped optional services:
  - object storage when blob/media scope is enabled
  - coturn when voice/TURN validation or constrained-network media scope is enabled
- API secrets/config that must be reviewed explicitly:
  - `API_DATABASE_URL`
  - `API_SESSION_SIGNING_KEYS`
  - `API_SESSION_SIGNING_KEY_ID`
  - `API_CHANNEL_DISPATCH_INTERNAL_TOKEN`
  - `API_PRESENCE_WATCHER_INTERNAL_TOKEN`
- Realtime secrets/config that must be reviewed explicitly:
  - `REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN`
  - `REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN`
- Redis URLs remain optional at pure config-validation time, but they are required for the reviewed dedicated single-server deployment baseline.
- Static peer descriptor and invite JSON are optional at pure config-validation time. When set, the service rejects startup on malformed JSON, invalid descriptor or invite policy, expired descriptors/invites, over-TTL descriptors/invites, duplicate server/descriptor IDs, revoked invite IDs, or invalid Ed25519 signatures.
- API local server identity JSON/key config is optional for local-only operation. When set, both values are required, the descriptor is signature/TTL/policy validated, descriptor `server_id` must match `API_SERVER_ID`, and the private key must derive the descriptor public key.
- `API_SERVER_OWNER_IDENTITY_IDS` and `API_SERVER_ADMIN_IDENTITY_IDS` are bootstrap allowlists for the app-mediated dedicated-server administration contract. They must contain valid identity ids only, do not grant access without normal session authentication, and should be replaced or backed by durable server-local roles once role-management flows are implemented.
- Dedicated deployments should also review:
  - origin allowlists
  - proxy-header trust flags
  - cookie security/domain settings
  - auth grace/cache settings
  - server owner/admin bootstrap allowlists
  - local server-to-server signing descriptor/key source and rotation process
  - static private-mesh descriptor/invite source and revocation process

## Change Rule

- If `services/api-rs/src/config.rs`, `services/realtime-rs/src/config.rs`, `services/api-rs/.env.example`, or `services/realtime-rs/.env.example` changes, update this document and `docs/README.md` in the same PR.

## Related Documents

- `docs/architecture/01-system-overview.md`
- `docs/operations/local-runtime-testing-quickstart.md`
- `docs/operations/dev-prerequisites.md`
- `docs/operations/01-mvp-runbook.md`
- `docs/product/09-configuration-defaults-register.md`
