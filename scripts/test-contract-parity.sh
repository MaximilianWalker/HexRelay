#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
SCRIPT_PATH="$ROOT_DIR/scripts/validate-contract-parity.sh"
FIXTURES_DIR="$ROOT_DIR/scripts/fixtures/contract-parity"
FIXTURE_GIT_AUTHOR_NAME="OpenCode Fixture"
FIXTURE_GIT_AUTHOR_EMAIL="fixture@hexrelay.local"

if command -v py >/dev/null 2>&1; then
  PYTHON_BIN=(py -3)
elif command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN=(python3)
elif command -v python >/dev/null 2>&1; then
  PYTHON_BIN=(python)
else
  echo "::error::python3, python, or py -3 is required for contract parity fixture mutations."
  exit 1
fi

run_fixture() {
  local fixture_name="$1"
  local expected_exit="$2"
  local expected_text="${3:-}"
  local fixture_dir="$FIXTURES_DIR/$fixture_name"
  local temp_repo
  temp_repo="$(mktemp -d)"
  trap 'rm -rf "$temp_repo"' RETURN

  cp -R "$fixture_dir/." "$temp_repo/"
  cp "$ROOT_DIR/.gitattributes" "$temp_repo/.gitattributes"
  git -C "$temp_repo" init -q
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit --allow-empty -qm "base"
  git -C "$temp_repo" add .
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit -qm "fixture"

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

run_response_header_schema_type_fixture() {
  local mutation_name="$1"
  local expected_exit="$2"
  local expected_text="${3:-}"
  local fixture_dir="$FIXTURES_DIR/pass-cookie-actions"
  local temp_repo
  temp_repo="$(mktemp -d)"
  trap 'rm -rf "$temp_repo"' RETURN

  cp -R "$fixture_dir/." "$temp_repo/"
  cp "$ROOT_DIR/.gitattributes" "$temp_repo/.gitattributes"
  git -C "$temp_repo" init -q
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit --allow-empty -qm "base"
  "${PYTHON_BIN[@]}" - "$temp_repo/docs/contracts/runtime-rest.openapi.yaml" "$mutation_name" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
mutation_name = sys.argv[2]
text = path.read_text()
old = "schema:\n                type: string"
if mutation_name == "fail-response-header-schema-type":
    new = "schema:\n                type: integer"
elif mutation_name == "pass-response-header-schema-ref":
    new = "schema:\n                $ref: '#/components/schemas/CookieHeader'"
    text += "\n    CookieHeader:\n      type: string\n"
else:
    raise SystemExit(f"unknown fixture mutation: {mutation_name}")
if old not in text:
    raise SystemExit("fixture mutation target not found")
path.write_text(text.replace(old, new, 1))
PY
  git -C "$temp_repo" add .
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit -qm "fixture"

  set +e
  local output
  output="$(cd "$temp_repo" && bash "$SCRIPT_PATH" HEAD~1 HEAD 2>&1)"
  local exit_code=$?
  set -e

  if [ "$exit_code" -ne "$expected_exit" ]; then
    printf 'fixture %s: expected exit %s, got %s\n%s\n' "$mutation_name" "$expected_exit" "$exit_code" "$output"
    return 1
  fi

  if [ -n "$expected_text" ] && ! printf '%s' "$output" | grep -Fq "$expected_text"; then
    printf 'fixture %s: expected output to contain %s\n%s\n' "$mutation_name" "$expected_text" "$output"
    return 1
  fi

  rm -rf "$temp_repo"
  trap - RETURN
}

run_path_parameter_format_fixture() {
  local mutation_name="$1"
  local expected_exit="$2"
  local expected_text="${3:-}"
  local fixture_dir="$FIXTURES_DIR/pass-basic"
  local temp_repo
  temp_repo="$(mktemp -d)"
  trap 'rm -rf "$temp_repo"' RETURN

  cp -R "$fixture_dir/." "$temp_repo/"
  cp "$ROOT_DIR/.gitattributes" "$temp_repo/.gitattributes"
  git -C "$temp_repo" init -q
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit --allow-empty -qm "base"
  "${PYTHON_BIN[@]}" - "$temp_repo/services/api-rs/src/transport/http/handlers/friends.rs" "$temp_repo/docs/contracts/runtime-rest.openapi.yaml" "$mutation_name" <<'PY'
import pathlib
import sys

handler_path = pathlib.Path(sys.argv[1])
contract_path = pathlib.Path(sys.argv[2])
mutation_name = sys.argv[3]

handler_text = handler_path.read_text()
old_handler = "Path(_request_id): Path<String>,"
new_handler = "Path(_request_id): Path<uuid::Uuid>,"
if old_handler not in handler_text:
    raise SystemExit("fixture handler mutation target not found")
handler_path.write_text(handler_text.replace(old_handler, new_handler, 1))

if mutation_name == "pass-path-parameter-format":
    contract_text = contract_path.read_text()
    old_contract = "            type: string"
    new_contract = "            type: string\n            format: uuid"
    if old_contract not in contract_text:
        raise SystemExit("fixture contract mutation target not found")
    contract_path.write_text(contract_text.replace(old_contract, new_contract, 1))
elif mutation_name != "fail-path-parameter-format":
    raise SystemExit(f"unknown fixture mutation: {mutation_name}")
PY
  git -C "$temp_repo" add .
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit -qm "fixture"

  set +e
  local output
  output="$(cd "$temp_repo" && bash "$SCRIPT_PATH" HEAD~1 HEAD 2>&1)"
  local exit_code=$?
  set -e

  if [ "$exit_code" -ne "$expected_exit" ]; then
    printf 'fixture %s: expected exit %s, got %s\n%s\n' "$mutation_name" "$expected_exit" "$exit_code" "$output"
    return 1
  fi

  if [ -n "$expected_text" ] && ! printf '%s' "$output" | grep -Fq "$expected_text"; then
    printf 'fixture %s: expected output to contain %s\n%s\n' "$mutation_name" "$expected_text" "$output"
    return 1
  fi

  rm -rf "$temp_repo"
  trap - RETURN
}

run_fixture pass-basic 0
run_fixture pass-cookie-actions 0
run_path_parameter_format_fixture pass-path-parameter-format 0
run_response_header_schema_type_fixture pass-response-header-schema-ref 0
run_fixture pass-request-body-component 0
run_fixture pass-request-schema-alias 0
run_fixture pass-response-schema-alias 0
run_fixture pass-session-auth-security 0
run_fixture pass-server-channel-example-status 0
run_fixture fail-cookie-actions 1 "issue:hexrelay_csrf"
run_fixture fail-csrf-header-semantics 1 'enforces CSRF header `x-csrf-token` as type `string` at runtime but documents `integer`'
run_fixture fail-discovery-query-semantics 1 "default:global"
run_fixture fail-dm-control-example 1 "dm_policy_invalid"
run_fixture fail-error-response-schema 1 'can return HTTP 400 with ApiError at runtime but documents schema `FriendRequestRecord` instead of `ApiError`'
run_fixture fail-fanout-example 1 "fanout_invalid"
run_fixture fail-helper-auth-401 1 "can return HTTP 401 at runtime via direct unauthorized emitters or local failure helpers"
run_fixture fail-internal-auth-401 1 "requires internal-token auth at runtime but is missing a 401 response"
run_fixture fail-internal-auth-header 1 "x-hexrelay-internal-token"
run_fixture fail-internal-auth-header-semantics 1 'requires request header `x-hexrelay-internal-token` at runtime but it is not marked required'
run_fixture fail-internal-auth-security 1 "should not declare session security schemes"
run_fixture fail-internal-auth-example 1 "internal_token_invalid"
run_fixture fail-invite-create-example 1 "invite_invalid"
run_fixture fail-missing-csrf-header 1 "missing the CsrfTokenHeader parameter"
run_fixture fail-missing-request-body 1 "missing requestBody"
run_fixture fail-nonauth-helper-500 1 "local helper/delegate flows but is missing a 500 response"
run_fixture fail-no-content-success-schema 1 "returns HTTP 204 without a JSON success body"
run_path_parameter_format_fixture fail-path-parameter-format 1 'uses path parameter `request_id` with format `uuid` at runtime but documents `<none>`'
run_fixture fail-path-parameter-semantics 1 'uses path parameter `request_id` as type `string` at runtime but documents `integer`'
run_fixture fail-public-auth-security 1 'GET /health documents security schemes [BearerAuth, CookieAuth] but runtime does not require session or internal-token auth'
run_fixture fail-request-body-required 1 "requestBody is not marked required"
run_fixture fail-request-schema-ref-direct 1 "FriendRequestCreateRequest"
run_fixture fail-request-schema-ref-alias 1 "FriendRequestCreateRequest"
run_fixture fail-rest-schema-field-types 1 'uses request schema `AuthVerifyRequest` field `signature` as type `string` at runtime but documents `integer`'
run_fixture fail-rest-schema-array-item-ref 1 'returns schema `DmFanoutCatchUpResponse` field `items` array items as schema `DmFanoutCatchUpItem` at runtime but documents `FriendRequestPage`'
run_fixture fail-rest-schema-date-time-format 1 'returns schema `AuthVerifyResponse` field `expires_at` format `date-time` at runtime but documents `<none>`'
run_fixture fail-rest-schema-nullable-field 1 'uses request schema `DmFanoutCatchUpRequest` field `cursor` nullable `true` at runtime but documents `false`'
run_fixture fail-rest-schema-scalar-bounds 1 'uses request schema `DmFanoutCatchUpRequest` field `limit` maximum `100` at runtime but documents `50`'
run_fixture fail-rest-schema-enum-domain 1 'returns schema `DmFanoutCatchUpResponse` field `status` enum [blocked, ready] at runtime but documents [ready]'
run_fixture fail-rest-schema-string-pattern 1 'uses request schema `AuthVerifyRequest` field `identity_id` pattern `^[A-Za-z0-9_-]{3,64}$` at runtime but documents `<none>`'
run_fixture fail-rest-schema-nested-item-field-type 1 'returns schema `DmFanoutCatchUpResponse` field `items` array items reference schema `DmFanoutCatchUpItem` field `ciphertext` as type `string` at runtime but documents `integer`'
run_fixture fail-rest-schema-required-fields 1 'uses request schema `AuthVerifyRequest` with required fields [challenge_id, identity_id, signature] at runtime but documents [challenge_id, identity_id]'
run_fixture fail-realtime-error-envelope-semantics 1 'Realtime runtime event `error` uses data fields [code, message] but documents [code]'
run_fixture fail-realtime-envelope-semantics 1 'Realtime runtime event `realtime.connected` uses data fields [state] but documents [status]'
run_fixture fail-realtime-signal-envelope-semantics 1 'Realtime runtime event `call.signal.offer` uses data fields [call_id, from_identity_id, sdp_offer, to_identity_id] but documents [call_id, from_identity_id, to_identity_id]'
run_fixture fail-realtime-signaling-semantics 1 'Realtime runtime event `call.signal.offer` requires from_identity_id/session-identity parity at runtime but does not require it'
run_fixture fail-response-header 1 'returns response header `Set-Cookie` for HTTP 200 at runtime but is missing it'
run_response_header_schema_type_fixture fail-response-header-schema-type 1 'returns response header `Set-Cookie` for HTTP 200 as type `string` at runtime but documents `integer`'
run_fixture fail-response-schema-ref 1 "PresenceWatcherListResponse"
run_fixture fail-server-channel-example-status 1 "missing tracked HTTP 400 route-level error examples for ApiError codes [reply_target_invalid]"
run_fixture fail-session-auth-401 1 "missing a 401 response"
run_fixture fail-session-auth-security 1 "documents security schemes [CookieAuth] instead of [BearerAuth, CookieAuth]"
run_fixture fail-session-auth-500 1 "missing a 500 response"
run_fixture fail-success-content 1 "documents no success schema"
run_fixture fail-unexpected-request-body 1 "documents a requestBody but runtime handler has no request-body extractor"
run_fixture fail-missing-example 1 "thread_not_found"

printf '[contract-parity-test] Fixture regressions passed\n'
