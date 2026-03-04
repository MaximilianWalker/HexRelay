# HexRelay Configuration Defaults Register

## Document Metadata

- Doc ID: config-defaults-register
- Owner: Product and platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/product/09-configuration-defaults-register.md`

## Quick Context

- Purpose: define default values, allowed ranges, and override precedence for MVP policies.
- Primary edit location: update when policy defaults or override rules change.
- Latest meaningful change: 2026-03-04 execution-hardening pass added explicit defaults register.

## Override Precedence

- Effective precedence: `node_policy` > `server_policy` > `user_policy` > `device_preference`.
- If policies conflict, stricter privacy/security rule wins.

## Defaults

| Key | Default | Allowed values | Scope |
|---|---|---|---|
| `invite.mode` | `multi_use` | `one_time`, `multi_use` | server |
| `invite.expires_at` | `null` | ISO date-time or `null` | server |
| `invite.max_uses` | `null` | integer >= 1 or `null` | server |
| `contact_invite.mode` | `one_time` | `one_time`, `multi_use` | user |
| `contact_invite.expires_at` | `24h` | ISO date-time | user |
| `contact_invite.max_uses` | `1` | integer >= 1 | user |
| `dm.inbound_policy` | `friends_only` | `friends_only`, `same_server`, `anyone` | user |
| `dm.offline_delivery_mode` | `best_effort_online` | `best_effort_online` | user |
| `discovery.listing_visibility` | `private` | `private`, `public` | server |
| `storage.quota_mb` | `null` | integer >= 100 or `null` | server |
| `retention.message_days` | `null` | integer >= 1 or `null` | server |
| `ui.server_nav_mode` | `sidebar` | `sidebar`, `topbar` | device |
| `ui.server_nav_visibility` | `expanded` | `expanded`, `collapsed`, `hidden` | device |

## Related Documents

- `docs/product/02-prd-v1.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/planning/iterations/02-sprint-board.md`
