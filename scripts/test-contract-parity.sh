#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
SCRIPT_PATH="$ROOT_DIR/scripts/validate-contract-parity.sh"
FIXTURES_DIR="$ROOT_DIR/scripts/fixtures/contract-parity"

run_fixture() {
  local fixture_name="$1"
  local expected_exit="$2"
  local expected_text="${3:-}"
  local fixture_dir="$FIXTURES_DIR/$fixture_name"
  local temp_repo
  temp_repo="$(mktemp -d)"
  trap 'rm -rf "$temp_repo"' RETURN

  cp -R "$fixture_dir/." "$temp_repo/"
  git -C "$temp_repo" init -q
  git -C "$temp_repo" commit --allow-empty -qm "base"
  git -C "$temp_repo" add .
  git -C "$temp_repo" commit -qm "fixture"

  set +e
  local output
  output="$(cd "$temp_repo" && bash "$SCRIPT_PATH" HEAD~1 HEAD 2>&1)"
  local exit_code=$?
  set -e

  if [ "$exit_code" -ne "$expected_exit" ]; then
    printf 'fixture %s: expected exit %s, got %s\n%s\n' "$fixture_name" "$expected_exit" "$exit_code" "$output"
    return 1
  fi

  if [ -n "$expected_text" ] && ! printf '%s' "$output" | grep -Fq "$expected_text"; then
    printf 'fixture %s: expected output to contain %s\n%s\n' "$fixture_name" "$expected_text" "$output"
    return 1
  fi

  rm -rf "$temp_repo"
  trap - RETURN
}

run_fixture pass-basic 0
run_fixture fail-missing-example 1 "thread_not_found"

printf '[contract-parity-test] Fixture regressions passed\n'
