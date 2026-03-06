#!/usr/bin/env bash
set -euo pipefail

base_sha="${1:-}"
head_sha="${2:-HEAD}"

if [ -z "${base_sha}" ] || [ "${base_sha}" = "0000000000000000000000000000000000000000" ]; then
  base_sha="$(git merge-base HEAD origin/master 2>/dev/null || git rev-parse HEAD~1)"
fi

migration_files="$(git diff --name-only "${base_sha}" "${head_sha}" -- 'services/api-rs/migrations/*.sql')"

if [ -z "${migration_files}" ]; then
  echo "[migration-evidence] No migration SQL changes detected"
  exit 0
fi

missing=0
while IFS= read -r migration_file; do
  [ -z "${migration_file}" ] && continue
  migration_name="$(basename "${migration_file}" .sql)"
  evidence_file="evidence/migrations/${migration_name}.md"

  if ! git diff --name-only "${base_sha}" "${head_sha}" -- "${evidence_file}" | grep -q "${evidence_file}"; then
    echo "::error::Migration ${migration_file} changed but missing evidence artifact update at ${evidence_file}."
    missing=1
  fi
done <<< "${migration_files}"

if [ "${missing}" -ne 0 ]; then
  echo "[migration-evidence] Copy docs/operations/migration-validation-template.md into evidence/migrations/<migration>.md and fill it in."
  exit 1
fi

echo "[migration-evidence] Migration evidence artifacts are present for all changed migrations"
