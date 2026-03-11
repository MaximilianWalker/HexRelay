#!/usr/bin/env bash
set -euo pipefail

base_sha="${1:-}"
head_sha="${2:-HEAD}"

if [ -z "${base_sha}" ] || [ "${base_sha}" = "0000000000000000000000000000000000000000" ]; then
  base_sha="$(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD origin/master 2>/dev/null || git rev-parse HEAD~1)"
fi

api_contract="docs/contracts/runtime-rest-v1.openapi.yaml"
realtime_contract="docs/contracts/realtime-events-runtime-v1.asyncapi.yaml"

api_surface_files=(
  'services/api-rs/src/models.rs'
  'services/api-rs/src/app/router.rs'
  'services/api-rs/src/transport/http/middleware/auth.rs'
  'services/api-rs/src/transport/http/handlers/*.rs'
)

api_surface_changes="$(git diff --name-only "${base_sha}" "${head_sha}" -- "${api_surface_files[@]}")"

realtime_surface_files=(
  'services/realtime-rs/src/app/router.rs'
  'services/realtime-rs/src/domain/events/*.rs'
  'services/realtime-rs/src/transport/ws/handlers/*.rs'
)

realtime_surface_changes="$(git diff --name-only "${base_sha}" "${head_sha}" -- "${realtime_surface_files[@]}")"

api_contract_changed=0
if git diff --name-only "${base_sha}" "${head_sha}" -- "${api_contract}" | grep -qxF "${api_contract}"; then
  api_contract_changed=1
fi

realtime_contract_changed=0
if git diff --name-only "${base_sha}" "${head_sha}" -- "${realtime_contract}" | grep -qxF "${realtime_contract}"; then
  realtime_contract_changed=1
fi

errors=0

if [ -n "${api_surface_changes}" ] && [ "${api_contract_changed}" -ne 1 ]; then
  echo "::error::API contract-surface files changed but ${api_contract} was not updated."
  echo "Changed API surface files:"
  echo "${api_surface_changes}"
  errors=1
fi

if [ -n "${realtime_surface_changes}" ] && [ "${realtime_contract_changed}" -ne 1 ]; then
  echo "::error::Realtime websocket/event surface changed but ${realtime_contract} was not updated."
  echo "Changed realtime surface files:"
  echo "${realtime_surface_changes}"
  errors=1
fi

if ! grep -q '^openapi:' "${api_contract}"; then
  echo "::error::${api_contract} is missing required openapi version field."
  errors=1
fi

if ! grep -q '^asyncapi:' "${realtime_contract}"; then
  echo "::error::${realtime_contract} is missing required asyncapi version field."
  errors=1
fi

if [ "${errors}" -ne 0 ]; then
  echo "[contract-parity] Update runtime contract docs when API/realtime surface changes."
  exit 1
fi

echo "[contract-parity] Runtime contract parity checks passed"
