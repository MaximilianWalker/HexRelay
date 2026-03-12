#!/usr/bin/env bash
set -euo pipefail

advisory_id="RUSTSEC-2023-0071"
ignore_expiry_utc="2026-06-30"

today_utc="$(date -u +%F)"

if [[ "${today_utc}" > "${ignore_expiry_utc}" ]]; then
  echo "::error::cargo-audit ignore ${advisory_id} expired on ${ignore_expiry_utc}."
  echo "Remove the ignore or renew with explicit rationale in the same PR."
  exit 1
fi

echo "[security] cargo-audit ignore ${advisory_id} valid until ${ignore_expiry_utc}"
