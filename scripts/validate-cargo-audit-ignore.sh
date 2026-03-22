#!/usr/bin/env bash
set -euo pipefail

declare -A advisories=(
  ["RUSTSEC-2023-0071"]="2026-06-30"
  ["RUSTSEC-2026-0049"]="2026-09-30"
)

today_utc="$(date -u +%F)"
failed=0

for advisory_id in "${!advisories[@]}"; do
  ignore_expiry_utc="${advisories[$advisory_id]}"
  if [[ "${today_utc}" > "${ignore_expiry_utc}" ]]; then
    echo "::error::cargo-audit ignore ${advisory_id} expired on ${ignore_expiry_utc}."
    echo "Remove the ignore or renew with explicit rationale in the same PR."
    failed=1
  else
    echo "[security] cargo-audit ignore ${advisory_id} valid until ${ignore_expiry_utc}"
  fi
done

exit "${failed}"
