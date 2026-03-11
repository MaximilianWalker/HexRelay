#!/usr/bin/env bash
set -euo pipefail

base_sha="${1:-}"
head_sha="${2:-HEAD}"

if [ -z "${base_sha}" ] || [ "${base_sha}" = "0000000000000000000000000000000000000000" ]; then
  base_sha="$(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD origin/master 2>/dev/null || git rev-parse HEAD~1)"
fi

index_file="docs/README.md"

docs_changes="$(git diff --name-only "${base_sha}" "${head_sha}" -- \
  'docs/**/*.md' \
  'docs/**/*.yaml' \
  'docs/**/*.yml' \
  'docs/**/*.json')"

canonical_changes="$(printf '%s\n' "${docs_changes}" | grep -Ev '^$|^docs/README\.md$' || true)"

if [ -z "${canonical_changes}" ]; then
  echo "[docs-index-freshness] No canonical docs changes detected"
  exit 0
fi

if ! git diff --name-only "${base_sha}" "${head_sha}" -- "${index_file}" | grep -qxF "${index_file}"; then
  echo "::error::Canonical docs changed but ${index_file} was not updated."
  echo "Changed canonical docs:"
  echo "${canonical_changes}"
  exit 1
fi

if ! grep -q '^- last_updated:' "${index_file}"; then
  echo "::error::${index_file} is missing metadata field: last_updated."
  exit 1
fi

echo "[docs-index-freshness] docs index updated with canonical docs changes"
