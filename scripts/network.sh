#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NODE_BIN="${NODE_BIN:-}"
if [[ -z "$NODE_BIN" ]]; then
  if command -v node >/dev/null 2>&1; then
    NODE_BIN="node"
  elif command -v node.exe >/dev/null 2>&1; then
    NODE_BIN="node.exe"
  elif [[ -x "/mnt/c/Program Files/nodejs/node.exe" ]]; then
    NODE_BIN="/mnt/c/Program Files/nodejs/node.exe"
  else
    echo "[network] ERROR: node was not found on PATH" >&2
    exit 1
  fi
fi

SCRIPT_PATH="$ROOT/scripts/network/index.mjs"
if [[ "$NODE_BIN" == *node.exe ]]; then
  if command -v wslpath >/dev/null 2>&1; then
    SCRIPT_PATH="$(wslpath -w "$SCRIPT_PATH")"
  elif command -v cygpath >/dev/null 2>&1; then
    SCRIPT_PATH="$(cygpath -w "$SCRIPT_PATH")"
  fi
fi

"$NODE_BIN" "$SCRIPT_PATH" "$@"
