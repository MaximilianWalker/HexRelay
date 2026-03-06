#!/usr/bin/env bash
set -euo pipefail

if [ ! -f "infra/.env" ]; then
  echo "[setup] Creating infra/.env from infra/.env.example"
  cp "infra/.env.example" "infra/.env"
fi

echo "[setup] Installing web dependencies"
npm install --prefix "apps/web"

echo "[setup] Fetching Rust dependencies"
cargo fetch --manifest-path "services/api-rs/Cargo.toml"
cargo fetch --manifest-path "services/realtime-rs/Cargo.toml"

echo "[setup] Installing pinned security tooling"
bash "scripts/ensure-cargo-audit.sh"

echo "[setup] Complete"
