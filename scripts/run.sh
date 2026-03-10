#!/usr/bin/env bash
set -euo pipefail

if [ ! -f "infra/.env" ]; then
  echo "[run] infra/.env missing; creating from infra/.env.example"
  cp "infra/.env.example" "infra/.env"
fi

echo "[run] Starting local infrastructure"
docker compose --env-file "infra/.env" -f "infra/docker-compose.yml" up -d

set -a
source "infra/.env"
set +a

wait_for() {
  local label="$1"
  shift
  local attempts="${WAIT_FOR_ATTEMPTS:-60}"
  local sleep_seconds="${WAIT_FOR_SLEEP_SECONDS:-1}"

  for ((i = 1; i <= attempts; i++)); do
    if "$@"; then
      echo "[run] ${label} is ready"
      return 0
    fi
    sleep "$sleep_seconds"
  done

  echo "[run] ERROR: ${label} did not become ready after ${attempts} attempts"
  return 1
}

echo "[run] Waiting for infrastructure health"
wait_for "postgres" docker compose --env-file infra/.env -f infra/docker-compose.yml exec -T postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"
wait_for "redis" docker compose --env-file infra/.env -f infra/docker-compose.yml exec -T redis redis-cli --raw ping
wait_for "minio" curl -fsS http://localhost:9000/minio/health/live

echo "[run] Starting API, Realtime, and Web dev servers"
cargo run -p api-rs &
API_PID=$!
cargo run -p realtime-rs &
RT_PID=$!
npm run dev --prefix "apps/web" &
WEB_PID=$!

cleanup() {
  echo "[run] Stopping local processes"
  kill "$API_PID" "$RT_PID" "$WEB_PID" 2>/dev/null || true
}

trap cleanup EXIT INT TERM

wait -n "$API_PID" "$RT_PID" "$WEB_PID"
