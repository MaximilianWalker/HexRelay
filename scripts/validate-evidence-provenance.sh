#!/usr/bin/env bash
set -euo pipefail

base_sha="${1:-}"
head_sha="${2:-HEAD}"

if [ -z "${base_sha}" ] || [ "${base_sha}" = "0000000000000000000000000000000000000000" ]; then
  base_sha="$(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD origin/master 2>/dev/null || git rev-parse HEAD~1)"
fi

changed_files="$(git diff --name-only "${base_sha}" "${head_sha}" -- 'evidence/iteration-*/**' 'evidence/operations/**')"

if [ -z "${changed_files}" ]; then
  echo "[evidence-provenance] No iteration/operations evidence changes detected"
  exit 0
fi

declare -A checked_dirs=()
missing=0

resolve_artifact_dir() {
  local path="$1"
  if [[ "${path}" == *"/outputs/"* ]]; then
    printf '%s\n' "${path%%/outputs/*}"
  else
    printf '%s\n' "${path%/*}"
  fi
}

while IFS= read -r changed_file; do
  [ -z "${changed_file}" ] && continue
  artifact_dir="$(resolve_artifact_dir "${changed_file}")"
  [ -z "${artifact_dir}" ] && continue

  if [ -n "${checked_dirs["${artifact_dir}"]+set}" ]; then
    continue
  fi
  checked_dirs["${artifact_dir}"]=1

  provenance_file="${artifact_dir}/provenance.json"
  if [ ! -f "${provenance_file}" ]; then
    echo "::error::Missing required provenance file at ${provenance_file}."
    missing=1
    continue
  fi

  if ! grep -q '"commit_sha"' "${provenance_file}"; then
    echo "::error::${provenance_file} missing required field commit_sha."
    missing=1
  fi

  if ! grep -q '"generated_at_utc"' "${provenance_file}"; then
    echo "::error::${provenance_file} missing required field generated_at_utc."
    missing=1
  fi

  if ! grep -q '"pr_number"' "${provenance_file}" && ! grep -q '"run_id"' "${provenance_file}"; then
    echo "::error::${provenance_file} must include pr_number or run_id."
    missing=1
  fi
done <<< "${changed_files}"

if [ "${missing}" -ne 0 ]; then
  echo "[evidence-provenance] Add provenance.json with commit_sha, generated_at_utc, and pr_number/run_id to each changed evidence artifact directory."
  exit 1
fi

echo "[evidence-provenance] Provenance files validated for changed evidence artifacts"
