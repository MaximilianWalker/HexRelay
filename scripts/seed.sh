#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

ensure_and_source_env() {
  local env_path="$1"
  local example_path="$2"
  local label="$3"

  if [ ! -f "$env_path" ]; then
    echo "[seed] $env_path missing; creating from $example_path"
    cp "$example_path" "$env_path"
  fi

  set -a
  source "$env_path"
  set +a

  echo "[seed] Loaded ${label} env from $env_path"
}

ensure_and_source_env "infra/.env" "infra/.env.example" "infra"
ensure_and_source_env "services/api-rs/.env" "services/api-rs/.env.example" "api"

cargo run -p api-rs --bin seed_dev -- "$@"
