#!/usr/bin/env bash
set -euo pipefail

RUN_ID="${GITHUB_RUN_ID:-local-$(date +%Y%m%d-%H%M%S)}"
EVIDENCE_DIR="evidence/ci/${RUN_ID}"

mkdir -p "${EVIDENCE_DIR}"

if [ -f api.log ]; then
  cp api.log "${EVIDENCE_DIR}/api.log"
fi

if [ -f realtime.log ]; then
  cp realtime.log "${EVIDENCE_DIR}/realtime.log"
fi

if [ -f smoke-e2e.log ]; then
  cp smoke-e2e.log "${EVIDENCE_DIR}/smoke-e2e.log"
fi

if [ -f health-checks.log ]; then
  cp health-checks.log "${EVIDENCE_DIR}/health-checks.log"
fi

cat > "${EVIDENCE_DIR}/manifest.txt" <<EOF
run_id=${RUN_ID}
collected_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
api_log=$(test -f "${EVIDENCE_DIR}/api.log" && echo present || echo missing)
realtime_log=$(test -f "${EVIDENCE_DIR}/realtime.log" && echo present || echo missing)
smoke_log=$(test -f "${EVIDENCE_DIR}/smoke-e2e.log" && echo present || echo missing)
health_checks=$(test -f "${EVIDENCE_DIR}/health-checks.log" && echo present || echo missing)
EOF

echo "Collected CI evidence in ${EVIDENCE_DIR}"
