# HexRelay Configuration Defaults Register

## Document Metadata

- Doc ID: config-defaults-register
- Owner: Product and platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/product/09-configuration-defaults-register.md`

## Quick Context

- Purpose: define default values, allowed ranges, and override precedence for MVP policies.
- Primary edit location: update when policy defaults or override rules change.
- Latest meaningful change: 2026-05-11 added DM delivery metadata retention and abuse-control defaults for server-to-server P2P encrypted-envelope delivery.

## Override Precedence

- Effective precedence: `server_policy` > `user_policy` > `device_preference`.
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
| `dm.delivery.mode` | `encrypted_envelope_server` | `encrypted_envelope_server` | user |
| `dm.delivery.server_payload` | `ciphertext_envelopes_only` | `ciphertext_envelopes_only` | user |
| `dm.delivery.metadata_scope` | `minimal_delivery_metadata` | `minimal_delivery_metadata` | user |
| `dm.offline_delivery_mode` | `encrypted_envelope_catchup` | `encrypted_envelope_catchup` | user |
| `dm.bootstrap.mode` | `signed_relationship_bootstrap` | `signed_relationship_bootstrap` | user |
| `dm.device_sync.active_fanout` | `all_active_devices` | `all_active_devices` | user |
| `dm.device_sync.catchup_mode` | `cursor_replay` | `cursor_replay` | user |
| `dm.delivery_metadata.retention_seconds` | `2592000` | integer >= 1 | deployment |
| `dm.outbound_forwarding_metadata.retention_seconds` | `604800` | integer >= 1 | deployment |
| `dm.rate_limit.dispatch_per_window` | `120` | integer >= 1 | deployment |
| `dm.rate_limit.catch_up_per_window` | `120` | integer >= 1 | deployment |
| `dm.rate_limit.ack_per_window` | `600` | integer >= 1 | deployment |
| `dm.rate_limit.internal_forward_per_window` | `240` | integer >= 1 | deployment |
| `dm.device_sync.replay_retention_hours` | derived from `dm.delivery_metadata.retention_seconds` | integer >= 1 | deployment |
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

## Runtime Environment Mapping

| Runtime env var | Product key | Default |
|---|---|---|
| `API_DM_DISPATCH_RATE_LIMIT` | `dm.rate_limit.dispatch_per_window` | `120` |
| `API_DM_CATCH_UP_RATE_LIMIT` | `dm.rate_limit.catch_up_per_window` | `120` |
| `API_DM_ACK_RATE_LIMIT` | `dm.rate_limit.ack_per_window` | `600` |
| `API_DM_INTERNAL_FORWARD_RATE_LIMIT` | `dm.rate_limit.internal_forward_per_window` | `240` |
| `API_DM_DELIVERY_LOG_RETENTION_SECONDS` | `dm.delivery_metadata.retention_seconds` | `2592000` |
| `API_DM_OUTBOUND_FORWARDING_LOG_RETENTION_SECONDS` | `dm.outbound_forwarding_metadata.retention_seconds` | `604800` |

## Related Documents

- `docs/product/02-prd.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/planning/iterations/02-sprint-board.md`
