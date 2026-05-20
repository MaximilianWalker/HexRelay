#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[test] Rust fmt/clippy/test"
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

echo "[test] Web lint/test/build"
npm run lint --prefix "apps/web"
npm run test:coverage --prefix "apps/web"
npm run build --prefix "apps/web"

echo "[test] Complete"
