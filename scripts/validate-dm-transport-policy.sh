#!/usr/bin/env bash
set -euo pipefail

runtime_forbidden_pattern='\b(stun|turn|coturn|webrtc|ice_server|dm_relay|relay_fallback)\b'
config_forbidden_pattern='\b(dm_.*(stun|turn|relay)|dm_relay|relay_fallback|ice_server)\b'
api_dm_validation_allowlist='DM_ENDPOINT_HINT_FORBIDDEN_SCHEMES|endpoint hints must not use relay-oriented schemes|turn://relay\.example\.com:3478|stun://192\.168\.1\.11:3478'

matches=""

core_matches="$(grep -RInEi "${runtime_forbidden_pattern}" --include='*.rs' "crates/communication-core/src" || true)"
if [ -n "${core_matches}" ]; then
  matches+="${core_matches}"$'\n'
fi

api_dm_matches="$(grep -RInEi "${runtime_forbidden_pattern}" --include='*.rs' "services/api-rs/src" | grep -viE "${api_dm_validation_allowlist}" || true)"
if [ -n "${api_dm_matches}" ]; then
  matches+="${api_dm_matches}"$'\n'
fi

config_matches="$(grep -InEi "${config_forbidden_pattern}" \
  ".github/workflows/ci.yml" \
  "docs/reference/runtime-config-reference.md" \
  "services/api-rs/src/config.rs" \
  "services/realtime-rs/src/config.rs" \
  || true)"
if [ -n "${config_matches}" ]; then
  matches+="${config_matches}"$'\n'
fi

if [ -n "${matches}" ]; then
  echo "::error::Detected forbidden DM infrastructure terms in runtime or config/workflow surfaces."
  echo "These terms are disallowed for DM direct transport policy (no STUN/TURN/relay fallback):"
  printf '%s' "${matches}"
  exit 1
fi

echo "[dm-transport-policy] Runtime and config/workflow surfaces passed direct-only DM policy guardrail"
