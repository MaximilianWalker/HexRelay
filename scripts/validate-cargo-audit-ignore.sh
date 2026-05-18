#!/usr/bin/env bash
set -euo pipefail

node_cmd=""
if command -v node >/dev/null 2>&1; then
  node_cmd="node"
elif command -v node.exe >/dev/null 2>&1; then
  node_cmd="node.exe"
elif command -v where.exe >/dev/null 2>&1; then
  node_windows="$(where.exe node.exe 2>/dev/null | tr -d '\r' | head -n 1)"
  if [ -n "${node_windows}" ]; then
    if command -v cygpath >/dev/null 2>&1; then
      node_cmd="$(cygpath -u "${node_windows}")"
    else
      node_cmd="${node_windows}"
    fi
  fi
fi

if [ -z "${node_cmd}" ]; then
  echo "::error::Node.js is required to validate cargo-audit ignore policy." >&2
  exit 1
fi

"${node_cmd}" scripts/cargo-audit-policy.mjs validate "$@"
