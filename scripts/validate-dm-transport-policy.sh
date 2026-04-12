#!/usr/bin/env bash
set -euo pipefail

runtime_forbidden_pattern='\b(stun|turn|coturn|webrtc|ice_server|dm_relay|relay_fallback)\b'
config_forbidden_pattern='\b(dm_.*(stun|turn|relay)|dm_relay|relay_fallback|ice_server)\b'

matches=""

core_matches="$(grep -RInE "${runtime_forbidden_pattern}" --include='*.rs' "crates/communication-core/src" || true)"
if [ -n "${core_matches}" ]; then
  matches+="${core_matches}"$'\n'
fi

api_dm_matches="$(grep -InE "${runtime_forbidden_pattern}" "services/api-rs/src/transport/http/handlers/dm.rs" || true)"
if [ -n "${api_dm_matches}" ]; then
  matches+="${api_dm_matches}"$'\n'
fi

config_matches="$(grep -InE "${config_forbidden_pattern}" \
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
