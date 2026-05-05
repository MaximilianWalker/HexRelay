#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_PATH="$ROOT/.local-run/runtime-state.json"
JSON=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json)
      JSON=1
      shift
      ;;
    --help|-h)
      echo "Usage: status.sh [--json]"
      exit 0
      ;;
    *)
      echo "[status] ERROR: unknown argument '$1'" >&2
      exit 1
      ;;
  esac
done

if [[ ! -f "$STATE_PATH" ]]; then
  if [[ "$JSON" -eq 1 ]]; then
    node -e 'console.log(JSON.stringify({active:false, instances:[]}, null, 2))'
  else
    echo "[status] No tracked local runtime is active."
  fi
  exit 0
fi

node --input-type=module - "$STATE_PATH" "$JSON" <<'NODE'
import fs from "node:fs";
import process from "node:process";

const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const asJson = process.argv[3] === "1";

function alive(pid) {
  try {
    process.kill(Number(pid), 0);
    return true;
  } catch {
    return false;
  }
}

async function httpOk(url) {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(3000) });
    return response.status === 200;
  } catch {
    return false;
  }
}

async function webReady(url) {
  return (await httpOk(url)) || (await httpOk(`${url.replace(/\/$/, "")}/onboarding/identity`));
}

const instances = [];
for (const instance of state.instances ?? []) {
  instances.push({
    ...instance,
    apiProcessAlive: alive(instance.apiPid),
    realtimeProcessAlive: alive(instance.realtimePid),
    webProcessAlive: alive(instance.webPid),
    apiHealthy: await httpOk(`${instance.apiUrl}/health`),
    realtimeHealthy: await httpOk(`${instance.realtimeUrl}/health`),
    webHealthy: await webReady(instance.webUrl),
  });
}

const result = { active: true, ...state, instances };
if (asJson) {
  console.log(JSON.stringify(result, null, 2));
  process.exit(0);
}

console.log(`[status] Runtime profile: ${result.profile}`);
if (result.seedProfile) console.log(`[status] Seed profile:    ${result.seedProfile}`);
console.log(`[status] Started at:      ${result.startedAt}`);
for (const instance of instances) {
  console.log("");
  console.log(`[${instance.id}]`);
  console.log(`  API:      pid=${instance.apiPid} process=${instance.apiProcessAlive} health=${instance.apiHealthy} ${instance.apiUrl}`);
  console.log(`  Realtime: pid=${instance.realtimePid} process=${instance.realtimeProcessAlive} health=${instance.realtimeHealthy} ${instance.realtimeUrl}`);
  console.log(`  Web:      pid=${instance.webPid} process=${instance.webProcessAlive} health=${instance.webHealthy} ${instance.webUrl}`);
  console.log(`  WS:       ${instance.realtimeWsUrl}`);
  console.log(`  Logs:     ${instance.logDir}`);
}
NODE
