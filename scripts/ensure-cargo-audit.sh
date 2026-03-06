#!/usr/bin/env bash
set -euo pipefail

REQUIRED_CARGO_AUDIT_VERSION="${CARGO_AUDIT_VERSION:-0.22.0}"

installed_version=""
if cargo audit --version >/dev/null 2>&1; then
  installed_version="$(cargo audit --version | sed -E 's/.* ([0-9]+\.[0-9]+\.[0-9]+).*/\1/')"
fi

if [ "${installed_version}" != "${REQUIRED_CARGO_AUDIT_VERSION}" ]; then
  echo "[security] Installing cargo-audit ${REQUIRED_CARGO_AUDIT_VERSION}"
  cargo install cargo-audit --version "${REQUIRED_CARGO_AUDIT_VERSION}" --locked
fi

echo "[security] Using $(cargo audit --version)"
