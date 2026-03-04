#!/usr/bin/env bash
set -euo pipefail

echo "[test] Rust fmt/clippy/test"
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

echo "[test] Web lint/test/build"
npm run lint --prefix "apps/web"
npm run test --prefix "apps/web"
npm run build --prefix "apps/web"

echo "[test] Complete"
