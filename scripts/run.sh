#!/usr/bin/env bash
set -euo pipefail

echo "[run] Start infrastructure first if needed: docker compose --env-file infra/.env up -d"
echo "[run] API service: cargo run -p api-rs"
echo "[run] Realtime service: cargo run -p realtime-rs"
echo "[run] Web app: npm run dev --prefix apps/web"
