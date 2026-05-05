#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_PATH="$ROOT/.local-run/runtime-state.json"
RUNTIME_PROFILE=""
JSON=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --runtime-profile|-RuntimeProfile|-r)
      RUNTIME_PROFILE="${2:-}"
      shift 2
      ;;
    --json)
      JSON=1
      shift
      ;;
    --help|-h)
      echo "Usage: stop.sh [--runtime-profile single|dual|triple|path] [--json]"
      exit 0
      ;;
    *)
      echo "[stop] ERROR: unknown argument '$1'" >&2
      exit 1
      ;;
  esac
done

if [[ ! -f "$STATE_PATH" ]]; then
  if [[ "$JSON" -eq 1 ]]; then
    node -e 'console.log(JSON.stringify({stopped:[], message:"no tracked local runtime is active"}, null, 2))'
  else
    echo "[stop] No tracked local runtime is active."
  fi
  exit 0
fi

pid_matches_launcher() {
  local pid="$1"
  local launcher="$2"
  local args
  args="$(ps -p "$pid" -o args= 2>/dev/null || true)"
  [[ -n "$launcher" && "$args" == *"$launcher"* ]]
}

stop_tree() {
  local target_pid="$1"
  local child
  while IFS= read -r child; do
    [[ -n "$child" ]] && stop_tree "$child"
  done < <(pgrep -P "$target_pid" 2>/dev/null || true)
  kill -TERM "$target_pid" >/dev/null 2>&1 || true
  sleep 1
  kill -KILL "$target_pid" >/dev/null 2>&1 || true
}

ACTIVE_PROFILE="$(node -e 'const fs=require("node:fs"); const s=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); console.log(s.profile);' "$STATE_PATH")"
PROFILE_MATCH="1"
if [[ -n "$RUNTIME_PROFILE" ]]; then
  PROFILE_MATCH="$(node -e 'const fs=require("node:fs"); const cp=require("node:child_process"); const state=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); const spec=process.argv[2]; const validator=process.argv[3]; let ok=state.profile === spec || state.profilePath === spec; if (!ok) { const result=cp.spawnSync(process.execPath, [validator, "--print", spec], {encoding:"utf8"}); if (result.status === 0) { const resolved=JSON.parse(result.stdout); ok=state.profile === resolved.name || state.profilePath === resolved.profilePath; } } process.stdout.write(ok ? "1" : "0");' "$STATE_PATH" "$RUNTIME_PROFILE" "$ROOT/scripts/validate-runtime-profiles.mjs")"
fi
if [[ "$PROFILE_MATCH" != "1" ]]; then
  echo "[stop] ERROR: Active runtime profile is '$ACTIVE_PROFILE', not '$RUNTIME_PROFILE'." >&2
  exit 1
fi

STOPPED_JSON="[]"
while IFS=$'\t' read -r instance_id service pid launcher; do
  stopped=false
  if [[ "$pid" =~ ^[0-9]+$ ]] && kill -0 "$pid" >/dev/null 2>&1; then
    if pid_matches_launcher "$pid" "$launcher"; then
      stop_tree "$pid"
      stopped=true
    fi
  fi
  entry_json="$(node -e 'const [instanceId, service, pid, stopped] = process.argv.slice(1); process.stdout.write(JSON.stringify({instanceId, service, pid:Number(pid), stopped: stopped === "true"}));' "$instance_id" "$service" "$pid" "$stopped")"
  STOPPED_JSON="$(node -e 'const values=JSON.parse(process.argv[1]); values.push(JSON.parse(process.argv[2])); process.stdout.write(JSON.stringify(values));' "$STOPPED_JSON" "$entry_json")"
done < <(node -e 'const fs=require("node:fs"); const s=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); for (const i of s.instances ?? []) for (const [service, pid, launcher] of [["api", i.apiPid, i.apiLauncher], ["realtime", i.realtimePid, i.realtimeLauncher], ["web", i.webPid, i.webLauncher]]) console.log([i.id, service, pid, launcher ?? ""].join("\t"));' "$STATE_PATH")

rm -f "$STATE_PATH"

if [[ "$JSON" -eq 1 ]]; then
  node -e 'console.log(JSON.stringify({profile:process.argv[1], stopped:JSON.parse(process.argv[2])}, null, 2))' "$ACTIVE_PROFILE" "$STOPPED_JSON"
  exit 0
fi

echo "[stop] Stopped tracked local runtime profile '$ACTIVE_PROFILE'."
node -e 'for (const entry of JSON.parse(process.argv[1])) console.log(`  [${entry.instanceId}] ${entry.service} pid=${entry.pid} stopped=${entry.stopped}`);' "$STOPPED_JSON"
