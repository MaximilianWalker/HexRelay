# HexRelay Configuration Defaults Register

## Document Metadata

- Doc ID: config-defaults-register
- Owner: Product and platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-16
- Source of truth: `docs/product/09-configuration-defaults-register.md`

## Quick Context

- Purpose: define default values, allowed ranges, and override precedence for MVP policies.
- Primary edit location: update when policy defaults or override rules change.
- Latest meaningful change: 2026-03-16 added profile-device sync defaults for active fanout and late-device catch-up convergence.

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
| `dm.connectivity.mode` | `direct_only` | `direct_only` | user |
| `dm.connectivity.bootstrap` | `oob_signed_envelope` | `oob_signed_envelope` | user |
| `dm.connectivity.fallback_behavior` | `fail_with_guidance` | `fail_with_guidance` | user |
| `dm.connectivity.lan_fast_path` | `enabled` | `enabled`, `disabled` | device |
| `dm.connectivity.wan_wizard` | `enabled` | `enabled`, `disabled` | device |
| `dm.connectivity.multi_endpoint_parallel_dial` | `enabled` | `enabled`, `disabled` | user |
| `dm.device_sync.active_fanout` | `all_active_devices` | `all_active_devices` | user |
| `dm.device_sync.catchup_mode` | `cursor_replay` | `cursor_replay` | user |
| `dm.device_sync.replay_retention_hours` | `72` | integer >= 1 | user |
| `server.device_sync.active_fanout` | `all_active_devices` | `all_active_devices` | server |
| `server.device_sync.catchup_mode` | `cursor_hydration` | `cursor_hydration` | server |
| `server.device_sync.replay_retention_hours` | `72` | integer >= 1 | server |
| `discovery.listing_visibility` | `private` | `private`, `public` | server |
| `storage.quota_mb` | `null` | integer >= 100 or `null` | server |
| `retention.message_days` | `null` | integer >= 1 or `null` | server |
| `ui.server_nav_mode` | `sidebar` | `sidebar`, `topbar` | device |
| `ui.server_nav_visibility` | `expanded` | `expanded`, `collapsed`, `hidden` | device |
| `runtime.mode` | `desktop_local` | `desktop_local`, `dedicated_server` | deployment |
| `runtime.desktop.start_local_services` | `true` | `true`, `false` | device |

## Related Documents

- `docs/product/02-prd-v1.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/planning/iterations/02-sprint-board.md`
