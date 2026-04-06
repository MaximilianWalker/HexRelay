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
run_fixture pass-cookie-actions 0
run_fixture pass-request-schema-alias 0
run_fixture fail-cookie-actions 1 "issue:hexrelay_csrf"
run_fixture fail-discovery-query-semantics 1 "default:global"
run_fixture fail-dm-control-example 1 "dm_policy_invalid"
run_fixture fail-fanout-example 1 "fanout_invalid"
run_fixture fail-helper-auth-401 1 "can return HTTP 401 at runtime via direct unauthorized emitters or local failure helpers"
run_fixture fail-internal-auth-401 1 "requires internal-token auth at runtime but is missing a 401 response"
run_fixture fail-internal-auth-header 1 "x-hexrelay-internal-token"
run_fixture fail-internal-auth-security 1 "should not declare session security schemes"
run_fixture fail-internal-auth-example 1 "internal_token_invalid"
run_fixture fail-invite-create-example 1 "invite_invalid"
run_fixture fail-request-schema-ref-alias 1 "FriendRequestCreateRequest"
run_fixture fail-response-schema-ref 1 "PresenceWatcherListResponse"
run_fixture fail-session-auth-401 1 "missing a 401 response"
run_fixture fail-session-auth-500 1 "missing a 500 response"
run_fixture fail-success-content 1 "documents no success schema"
run_fixture fail-missing-example 1 "thread_not_found"

printf '[contract-parity-test] Fixture regressions passed\n'
