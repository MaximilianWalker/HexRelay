#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RUNTIME_PROFILE="single"
SEED_PROFILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --runtime-profile|-RuntimeProfile|-r)
      RUNTIME_PROFILE="${2:-}"
      shift 2
      ;;
    --seed-profile|-SeedProfile)
      SEED_PROFILE="${2:-}"
      shift 2
      ;;
    --help|-h)
      echo "Usage: run.sh [--runtime-profile single|dual|triple|path] [--seed-profile dm-basic]"
      exit 0
      ;;
    *)
      echo "[run] ERROR: unknown argument '$1'" >&2
      exit 1
      ;;
  esac
done

ensure_and_source_env() {
  local env_path="$1"
  local example_path="$2"
  local label="$3"

  if [[ ! -f "$env_path" ]]; then
    echo "[run] $env_path missing; creating from $example_path"
    cp "$example_path" "$env_path"
  fi

  set -a
  # shellcheck source=/dev/null
  source "$env_path"
  set +a

  echo "[run] Loaded ${label} env from $env_path"
}

wait_for() {
  local label="$1"
  shift
  local attempts="${WAIT_FOR_ATTEMPTS:-60}"
  local sleep_seconds="${WAIT_FOR_SLEEP_SECONDS:-1}"

  for ((i = 1; i <= attempts; i++)); do
    if "$@" >/dev/null 2>&1; then
      echo "[run] ${label} is ready"
      return 0
    fi
    sleep "$sleep_seconds"
  done

  echo "[run] ERROR: ${label} did not become ready after ${attempts} attempts" >&2
  return 1
}

http_ok() {
  curl -fsS --max-time 5 "$1" >/dev/null 2>&1
}

web_ready() {
  local url="$1"
  http_ok "$url" || http_ok "${url%/}/onboarding/identity"
}

port_in_use() {
  local port="$1"
  (echo >/dev/tcp/127.0.0.1/"$port") >/dev/null 2>&1
}

is_reserved_port() {
  local port="$1"
  [[ " ${RESERVED_PORTS[*]} " == *" $port "* ]]
}

get_free_port() {
  local port="$1"
  while port_in_use "$port" || is_reserved_port "$port"; do
    if [[ "$port" -ge 65535 ]]; then
      echo "[run] ERROR: no available TCP port at or above $1" >&2
      return 1
    fi
    port=$((port + 1))
  done
  RESERVED_PORTS+=("$port")
  echo "$port"
}

pid_alive() {
  local pid="$1"
  [[ "$pid" =~ ^[0-9]+$ ]] && kill -0 "$pid" >/dev/null 2>&1
}

stop_tree() {
  local pid="$1"
  if pid_alive "$pid"; then
    local child
    while IFS= read -r child; do
      [[ -n "$child" ]] && stop_tree "$child"
    done < <(pgrep -P "$pid" 2>/dev/null || true)
    kill -TERM "$pid" >/dev/null 2>&1 || true
    sleep 1
    kill -KILL "$pid" >/dev/null 2>&1 || true
  fi
}

state_has_live_processes() {
  [[ -f "$STATE_PATH" ]] || return 1
  while IFS= read -r pid; do
    if pid_alive "$pid"; then
      return 0
    fi
  done < <(node -e 'const fs=require("node:fs"); const s=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); for (const i of s.instances ?? []) for (const p of [i.apiPid, i.realtimePid, i.webPid]) if (p) console.log(p);' "$STATE_PATH")
  return 1
}

append_instance_json() {
  local current="$1"
  local next="$2"
  node -e 'const instances=JSON.parse(process.argv[1]); instances.push(JSON.parse(process.argv[2])); process.stdout.write(JSON.stringify(instances));' "$current" "$next"
}

write_state() {
  node -e 'const fs=require("node:fs"); const state={profile:process.argv[2], profilePath:process.argv[3], seedProfile:process.argv[4] || null, infraMode:process.argv[5], startedAt:process.argv[6], root:process.argv[7], instances:JSON.parse(process.argv[8])}; fs.writeFileSync(process.argv[1], `${JSON.stringify(state, null, 2)}\n`);' "$STATE_PATH" "$PROFILE_NAME" "$PROFILE_PATH" "$SEED_PROFILE" "$INFRA_MODE" "$STARTED_AT" "$ROOT" "$STATE_INSTANCES_JSON"
}

cleanup() {
  if [[ ${#STARTED_PIDS[@]} -gt 0 ]]; then
    echo "[run] Stopping tracked local processes"
  fi
  for pid in "${STARTED_PIDS[@]:-}"; do
    stop_tree "$pid"
  done
  rm -f "$STATE_PATH"
}

ensure_and_source_env "infra/.env" "infra/.env.example" "infra"
ensure_and_source_env "services/api-rs/.env" "services/api-rs/.env.example" "api"
ensure_and_source_env "services/realtime-rs/.env" "services/realtime-rs/.env.example" "realtime"

PROFILE_JSON="$(node scripts/validate-runtime-profiles.mjs --print "$RUNTIME_PROFILE")"
PROFILE_NAME="$(node -e 'const p=JSON.parse(process.argv[1]); console.log(p.name)' "$PROFILE_JSON")"
PROFILE_PATH="$(node -e 'const p=JSON.parse(process.argv[1]); console.log(p.profilePath)' "$PROFILE_JSON")"
INFRA_MODE="$(node -e 'const p=JSON.parse(process.argv[1]); console.log(p.infraMode)' "$PROFILE_JSON")"
mapfile -t INSTANCE_ROWS < <(node -e 'const p=JSON.parse(process.argv[1]); for (const i of p.instances) console.log([i.id, i.apiPort, i.realtimePort, i.webPort, i.seedPersona ?? ""].join("\t"));' "$PROFILE_JSON")

RUN_DIR="$ROOT/.local-run"
STATE_PATH="$RUN_DIR/runtime-state.json"
mkdir -p "$RUN_DIR"

if state_has_live_processes; then
  echo "[run] ERROR: A tracked local runtime is already active. Run scripts/status.sh or scripts/stop.sh before starting another profile." >&2
  exit 1
fi
rm -f "$STATE_PATH"

echo "[run] Starting local infrastructure"
docker compose --env-file "infra/.env" -f "infra/docker-compose.yml" up -d postgres redis minio >/dev/null

echo "[run] Waiting for infrastructure health"
wait_for "postgres" docker compose --env-file infra/.env -f infra/docker-compose.yml exec -T postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"
wait_for "redis" docker compose --env-file infra/.env -f infra/docker-compose.yml exec -T redis redis-cli --raw ping
wait_for "minio" curl -fsS http://localhost:9000/minio/health/live

if [[ -n "$SEED_PROFILE" ]]; then
  echo "[run] Seeding local database with '$SEED_PROFILE'"
  cargo run -p api-rs --bin seed_dev -- --profile "$SEED_PROFILE" --json >"$RUN_DIR/seed.stdout.json" 2>"$RUN_DIR/seed.stderr.log"
  echo "[run] Seed output written to $RUN_DIR/seed.stdout.json"
fi

RESERVED_PORTS=()
STARTED_PIDS=()
STATE_INSTANCES_JSON="[]"
STARTED_AT="$(node -e 'console.log(new Date().toISOString())')"
declare -A FAILURES=()

trap cleanup EXIT INT TERM

for row in "${INSTANCE_ROWS[@]}"; do
  IFS=$'\t' read -r instance_id requested_api_port requested_realtime_port requested_web_port seed_persona <<< "$row"
  log_dir="$RUN_DIR/$instance_id"
  mkdir -p "$log_dir"

  api_port="$(get_free_port "$requested_api_port")"
  realtime_port="$(get_free_port "$requested_realtime_port")"
  web_port="$(get_free_port "$requested_web_port")"

  if [[ "$api_port" != "$requested_api_port" || "$realtime_port" != "$requested_realtime_port" || "$web_port" != "$requested_web_port" ]]; then
    echo "[run] $instance_id requested ports were unavailable; using api=$api_port realtime=$realtime_port web=$web_port"
  fi

  api_url="http://127.0.0.1:$api_port"
  realtime_url="http://127.0.0.1:$realtime_port"
  realtime_ws_url="ws://127.0.0.1:$realtime_port/ws"
  web_url="http://localhost:$web_port"
  allowed_origins="http://localhost:$web_port,http://127.0.0.1:$web_port"
  api_launcher="$log_dir/api-rs.sh"
  realtime_launcher="$log_dir/realtime-rs.sh"
  web_launcher="$log_dir/web.sh"

  echo "[run] Starting $instance_id API service"
  cat >"$api_launcher" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$ROOT"
export API_BIND="127.0.0.1:$api_port"
export API_REALTIME_BASE_URL="$realtime_url"
export API_ALLOWED_ORIGINS="$allowed_origins"
cargo run -p api-rs --bin api-rs
EOF
  bash "$api_launcher" >"$log_dir/api-rs.stdout.log" 2>"$log_dir/api-rs.stderr.log" &
  api_pid=$!
  STARTED_PIDS+=("$api_pid")
  wait_for "$instance_id api" http_ok "$api_url/health"

  echo "[run] Starting $instance_id realtime service"
  cat >"$realtime_launcher" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$ROOT"
export REALTIME_BIND="127.0.0.1:$realtime_port"
export REALTIME_API_BASE_URL="$api_url"
export REALTIME_ALLOWED_ORIGINS="$allowed_origins"
export REALTIME_ENABLE_DEV_FAULTS="true"
cargo run -p realtime-rs
EOF
  bash "$realtime_launcher" >"$log_dir/realtime-rs.stdout.log" 2>"$log_dir/realtime-rs.stderr.log" &
  realtime_pid=$!
  STARTED_PIDS+=("$realtime_pid")
  wait_for "$instance_id realtime" http_ok "$realtime_url/health"

  echo "[run] Starting $instance_id web dev server"
  runtime_tsconfig_dir="$ROOT/apps/web/.runtime-tsconfig"
  mkdir -p "$runtime_tsconfig_dir"
  cat >"$runtime_tsconfig_dir/$instance_id.json" <<EOF
{
  "extends": "../tsconfig.json",
  "include": [
    "../next-env.d.ts",
    "../**/*.ts",
    "../**/*.tsx",
    "../.next-$instance_id/types/**/*.ts",
    "../.next-$instance_id/dev/types/**/*.ts",
    "../**/*.mts"
  ],
  "exclude": ["../node_modules"]
}
EOF
  cat >"$web_launcher" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$ROOT/apps/web"
export HEXRELAY_RUNTIME_INSTANCE="$instance_id"
export NEXT_PUBLIC_API_BASE_URL="$api_url"
export NEXT_PUBLIC_REALTIME_WS_URL="$realtime_ws_url"
./node_modules/.bin/next dev --port "$web_port"
EOF
  bash "$web_launcher" >"$log_dir/web.stdout.log" 2>"$log_dir/web.stderr.log" &
  web_pid=$!
  STARTED_PIDS+=("$web_pid")
  wait_for "$instance_id web" web_ready "$web_url"

  realtime_internal_token="${REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN:-hexrelay-dev-channel-dispatch-token-change-me}"
  instance_json="$(node -e 'const [id, seedPersona, apiPort, realtimePort, webPort, apiPid, realtimePid, webPid, apiLauncher, realtimeLauncher, webLauncher, apiUrl, realtimeUrl, realtimeWsUrl, webUrl, logDir, realtimeInternalToken] = process.argv.slice(1); process.stdout.write(JSON.stringify({id, seedPersona: seedPersona || null, apiPort: Number(apiPort), realtimePort: Number(realtimePort), webPort: Number(webPort), apiPid: Number(apiPid), realtimePid: Number(realtimePid), webPid: Number(webPid), apiLauncher, realtimeLauncher, webLauncher, apiUrl, realtimeUrl, realtimeWsUrl, webUrl, logDir, realtimeInternalToken}));' "$instance_id" "$seed_persona" "$api_port" "$realtime_port" "$web_port" "$api_pid" "$realtime_pid" "$web_pid" "$api_launcher" "$realtime_launcher" "$web_launcher" "$api_url" "$realtime_url" "$realtime_ws_url" "$web_url" "$log_dir" "$realtime_internal_token")"
  STATE_INSTANCES_JSON="$(append_instance_json "$STATE_INSTANCES_JSON" "$instance_json")"
  write_state

  FAILURES["$instance_id:api"]=0
  FAILURES["$instance_id:realtime"]=0
  FAILURES["$instance_id:web"]=0
done

echo ""
echo "[run] Local runtime profile '$PROFILE_NAME' is ready"
node -e 'const instances=JSON.parse(process.argv[1]); for (const i of instances) { console.log(`  [${i.id}] API:      ${i.apiUrl}`); console.log(`  [${i.id}] Realtime: ${i.realtimeUrl}`); console.log(`  [${i.id}] WS:       ${i.realtimeWsUrl}`); console.log(`  [${i.id}] Web:      ${i.webUrl}`); console.log(`  [${i.id}] Logs:     ${i.logDir}`); }' "$STATE_INSTANCES_JSON"
echo ""
echo "[run] Use scripts/status.sh from another shell to inspect health."
echo "[run] Press Ctrl+C or run scripts/stop.sh to stop tracked processes."

while true; do
  while IFS=$'\t' read -r instance_id api_url realtime_url web_url; do
    for service in api realtime web; do
      key="$instance_id:$service"
      ok=1
      case "$service" in
        api) http_ok "$api_url/health" || ok=0 ;;
        realtime) http_ok "$realtime_url/health" || ok=0 ;;
        web) web_ready "$web_url" || ok=0 ;;
      esac

      if [[ "$ok" -eq 1 ]]; then
        FAILURES["$key"]=0
      else
        FAILURES["$key"]=$((FAILURES["$key"] + 1))
        if [[ "${FAILURES["$key"]}" -ge 15 ]]; then
          echo "[run] ERROR: $instance_id $service health check failed after startup" >&2
          exit 1
        fi
      fi
    done
  done < <(node -e 'const instances=JSON.parse(process.argv[1]); for (const i of instances) console.log([i.id, i.apiUrl, i.realtimeUrl, i.webUrl].join("\t"));' "$STATE_INSTANCES_JSON")

  sleep 2
done
