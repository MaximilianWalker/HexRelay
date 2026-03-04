#!/usr/bin/env bash
set -euo pipefail

echo "[setup] Installing web dependencies"
npm install --prefix "apps/web"

echo "[setup] Fetching Rust dependencies"
cargo fetch --manifest-path "services/api-rs/Cargo.toml"
cargo fetch --manifest-path "services/realtime-rs/Cargo.toml"

echo "[setup] Complete"
