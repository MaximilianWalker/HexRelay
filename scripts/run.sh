#!/usr/bin/env bash
set -euo pipefail

if [ ! -f "infra/.env" ]; then
  echo "[run] infra/.env missing; creating from infra/.env.example"
  cp "infra/.env.example" "infra/.env"
fi

echo "[run] Starting local infrastructure"
docker compose --env-file "infra/.env" -f "infra/docker-compose.yml" up -d

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
