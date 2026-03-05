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

if [ -f "apps/web/coverage/coverage-summary.json" ]; then
  cp "apps/web/coverage/coverage-summary.json" "${EVIDENCE_DIR}/web-coverage-summary.json"
fi

if command -v git >/dev/null 2>&1; then
  GIT_SHA="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
else
  GIT_SHA="unknown"
fi

cat > "${EVIDENCE_DIR}/manifest.txt" <<EOF
run_id=${RUN_ID}
collected_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
api_log=$(test -f "${EVIDENCE_DIR}/api.log" && echo present || echo missing)
realtime_log=$(test -f "${EVIDENCE_DIR}/realtime.log" && echo present || echo missing)
smoke_log=$(test -f "${EVIDENCE_DIR}/smoke-e2e.log" && echo present || echo missing)
health_checks=$(test -f "${EVIDENCE_DIR}/health-checks.log" && echo present || echo missing)
web_coverage_summary=$(test -f "${EVIDENCE_DIR}/web-coverage-summary.json" && echo present || echo missing)
EOF

{
  echo "{"
  echo "  \"run_id\": \"${RUN_ID}\"," 
  echo "  \"git_sha\": \"${GIT_SHA}\"," 
  echo "  \"collected_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," 
  echo "  \"artifacts\": ["
  first=1
  for file in api.log realtime.log smoke-e2e.log health-checks.log web-coverage-summary.json; do
    if [ -f "${EVIDENCE_DIR}/${file}" ]; then
      sha="$(sha256sum "${EVIDENCE_DIR}/${file}" | awk '{print $1}')"
      size="$(wc -c < "${EVIDENCE_DIR}/${file}")"
      if [ $first -eq 0 ]; then
        echo "    ,"
      fi
      first=0
      echo "    {\"file\": \"${file}\", \"sha256\": \"${sha}\", \"bytes\": ${size}}"
    fi
  done
  echo "  ]"
  echo "}"
} > "${EVIDENCE_DIR}/summary.json"

echo "Collected CI evidence in ${EVIDENCE_DIR}"
