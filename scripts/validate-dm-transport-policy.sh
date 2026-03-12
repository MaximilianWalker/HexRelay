#!/usr/bin/env bash
set -euo pipefail

forbidden_pattern='\b(stun|turn|coturn|webrtc|ice_server|ice_candidate|dm_relay|relay_fallback)\b'

matches=""

core_matches="$(grep -RInE "${forbidden_pattern}" --include='*.rs' "crates/communication-core/src" || true)"
if [ -n "${core_matches}" ]; then
  matches+="${core_matches}"$'\n'
fi

api_dm_matches="$(grep -RInE "${forbidden_pattern}" --include='*dm*.rs' "services/api-rs/src" || true)"
if [ -n "${api_dm_matches}" ]; then
  matches+="${api_dm_matches}"$'\n'
fi

realtime_dm_matches="$(grep -RInE "${forbidden_pattern}" --include='*dm*.rs' "services/realtime-rs/src" || true)"
if [ -n "${realtime_dm_matches}" ]; then
  matches+="${realtime_dm_matches}"$'\n'
fi

if [ -n "${matches}" ]; then
  echo "::error::Detected forbidden DM infrastructure terms in runtime source."
  echo "These terms are disallowed for DM direct transport policy (no STUN/TURN/relay fallback):"
  printf '%s' "${matches}"
  exit 1
fi

echo "[dm-transport-policy] Runtime source passed direct-only DM policy guardrail"
