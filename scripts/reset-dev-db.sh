#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

ensure_and_load_env() {
  local env_path="$1"
  local example_path="$2"
  local label="$3"

  if [ ! -f "$env_path" ]; then
    echo "[reset-dev-db] $env_path missing; creating from $example_path"
    cp "$example_path" "$env_path"
  fi

  while IFS= read -r line || [ -n "$line" ]; do
    line="${line#"${line%%[![:space:]]*}"}"
    line="${line%"${line##*[![:space:]]}"}"
    if [ -z "$line" ] || [[ "$line" == \#* ]]; then
      continue
    fi

    if [[ "$line" == export\ * ]]; then
      line="${line#export }"
    fi

    if [[ "$line" != *=* ]]; then
      continue
    fi

    local key="${line%%=*}"
    local value="${line#*=}"
    key="${key#"${key%%[![:space:]]*}"}"
    key="${key%"${key##*[![:space:]]}"}"
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"

    if [[ ! "$key" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; then
      echo "[reset-dev-db] ERROR: invalid env key '$key' in $env_path"
      exit 1
    fi

    if [[ "$value" == \"*\" && "$value" == *\" ]]; then
      value="${value:1:${#value}-2}"
    elif [[ "$value" == \'*\' && "$value" == *\' ]]; then
      value="${value:1:${#value}-2}"
    fi

    export "$key=$value"
  done < "$env_path"

  echo "[reset-dev-db] Loaded ${label} env from $env_path"
}

ensure_and_load_env "infra/.env" "infra/.env.example" "infra"
ensure_and_load_env "services/api-rs/.env" "services/api-rs/.env.example" "api"

cargo run -p api-rs --bin reset_dev_db -- "$@"
