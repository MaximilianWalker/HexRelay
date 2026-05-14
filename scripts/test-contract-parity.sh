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

run_response_builder_success_schema_fixture() {
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
if mutation_name != "fail-response-builder-success-schema":
    raise SystemExit(f"unknown fixture mutation: {mutation_name}")
old = "                $ref: '#/components/schemas/TestingSessionCreateResponse'"
new = "                $ref: '#/components/schemas/AuthVerifyResponse'"
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

run_api_error_schema_shape_fixture() {
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
  "${PYTHON_BIN[@]}" - "$temp_repo/docs/contracts/runtime-rest.openapi.yaml" "$mutation_name" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
mutation_name = sys.argv[2]
text = path.read_text()
if mutation_name != "fail-api-error-schema-shape":
    raise SystemExit(f"unknown fixture mutation: {mutation_name}")
old = """    ApiError:
      type: object
      required: [code, message]"""
new = """    ApiError:
      type: object
      required: [code]"""
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
    old_contract = """        - in: path
          name: request_id
          required: true
          schema:
            type: string"""
    new_contract = """        - in: path
          name: request_id
          required: true
          schema:
            type: string
            format: uuid"""
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

run_query_parameter_pattern_fixture() {
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
  "${PYTHON_BIN[@]}" - \
    "$temp_repo/services/api-rs/src/app/router.rs" \
    "$temp_repo/services/api-rs/src/models.rs" \
    "$temp_repo/services/api-rs/src/transport/http/handlers/friends.rs" \
    "$temp_repo/docs/contracts/runtime-rest.openapi.yaml" \
    "$mutation_name" <<'PY'
import pathlib
import sys

router_path = pathlib.Path(sys.argv[1])
models_path = pathlib.Path(sys.argv[2])
handler_path = pathlib.Path(sys.argv[3])
contract_path = pathlib.Path(sys.argv[4])
mutation_name = sys.argv[5]

if mutation_name != "fail-query-parameter-pattern":
    raise SystemExit(f"unknown fixture mutation: {mutation_name}")

router_text = router_path.read_text()
old_router = '        .route("/dm/threads", get(list_dm_threads));'
new_router = '''        .route("/dm/threads", get(list_dm_threads))
        .route("/friends/requests", get(list_friend_requests));'''
if old_router not in router_text:
    raise SystemExit("fixture router mutation target not found")
router_path.write_text(router_text.replace(old_router, new_router, 1))

models_path.write_text(models_path.read_text() + '''

pub struct FriendRequestListQuery {
    pub identity_id: String,
    pub direction: Option<String>,
}

pub struct FriendRequestPage {
    pub items: Vec<String>,
}
''')

handler_text = handler_path.read_text()
old_import = 'models::{DmThreadListQuery, DmThreadPage, FriendRequestAcceptRequest, FriendRequestRecord},'
new_import = 'models::{DmThreadListQuery, DmThreadPage, FriendRequestAcceptRequest, FriendRequestListQuery, FriendRequestPage, FriendRequestRecord},'
if old_import not in handler_text:
    raise SystemExit("fixture handler import mutation target not found")
handler_text = handler_text.replace(old_import, new_import, 1)
handler_text += '''

pub async fn list_friend_requests(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Query(query): Query<FriendRequestListQuery>,
) -> ApiResult<Json<FriendRequestPage>> {
    if false {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    let _identity_id = query.identity_id;
    let _direction = query.direction;
    Ok(Json(FriendRequestPage { items: vec![] }))
}
'''
handler_path.write_text(handler_text)

contract_text = contract_path.read_text()
insert_before = '  /dm/threads:\n'
friend_route = '''  /friends/requests:
    get:
      security:
        - CookieAuth: []
        - BearerAuth: []
      parameters:
        - in: query
          name: identity_id
          required: true
          schema:
            type: string
        - in: query
          name: direction
          schema:
            type: string
            enum: [inbound, outbound]
      responses:
        '200':
          description: Friend request page
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/FriendRequestPage'
        '400':
          description: Invalid friend request query
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                identityInvalid:
                  value:
                    code: identity_invalid
                    message: identity_id must be 3-64 chars using letters, numbers, _ or -
        '401':
          description: Unauthorized
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                identityInvalid:
                  value:
                    code: identity_invalid
                    message: identity_id must match authenticated session
        '500':
          $ref: '#/components/responses/InternalServerError'
'''
if insert_before not in contract_text:
    raise SystemExit("fixture contract mutation target not found")
contract_text = contract_text.replace(insert_before, friend_route + insert_before, 1)
contract_text = contract_text.replace(
    '    DmThreadPage:\n      type: object\n',
    '    DmThreadPage:\n      type: object\n    FriendRequestPage:\n      type: object\n',
    1,
)
contract_path.write_text(contract_text)
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

run_server_channel_request_schema_fixture() {
  local mutation_name="$1"
  local expected_exit="$2"
  local expected_text="${3:-}"
  local fixture_dir="$FIXTURES_DIR/pass-server-channel-example-status"
  local temp_repo
  temp_repo="$(mktemp -d)"
  trap 'rm -rf "$temp_repo"' RETURN

  cp -R "$fixture_dir/." "$temp_repo/"
  cp "$ROOT_DIR/.gitattributes" "$temp_repo/.gitattributes"
  git -C "$temp_repo" init -q
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit --allow-empty -qm "base"
  "${PYTHON_BIN[@]}" - \
    "$temp_repo/services/api-rs/src/models.rs" \
    "$temp_repo/docs/contracts/runtime-rest.openapi.yaml" \
    "$mutation_name" <<'PY'
import pathlib
import sys

models_path = pathlib.Path(sys.argv[1])
contract_path = pathlib.Path(sys.argv[2])
mutation_name = sys.argv[3]

models_text = models_path.read_text()
contract_text = contract_path.read_text()

if mutation_name == "fail-rest-schema-serde-default-required":
    old_model = "    pub mention_identity_ids: Option<Vec<String>>,"
    new_model = "    #[serde(default)]\n    pub mention_identity_ids: Vec<String>,"
    if old_model not in models_text:
        raise SystemExit("fixture model mutation target not found")
    models_path.write_text(models_text.replace(old_model, new_model, 1))

    old_contract = "      required: [content]"
    new_contract = "      required: [content, mention_identity_ids]"
    if old_contract not in contract_text:
        raise SystemExit("fixture contract required mutation target not found")
    contract_path.write_text(contract_text.replace(old_contract, new_contract, 1))
elif mutation_name == "fail-rest-schema-array-item-pattern":
    old_contract = "            pattern: '^[A-Za-z0-9_-]{3,64}$'"
    if old_contract not in contract_text:
        raise SystemExit("fixture contract item-pattern mutation target not found")
    contract_path.write_text(contract_text.replace(old_contract, "", 1))
else:
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

run_dm_mark_read_schema_fixture() {
  local mutation_name="$1"
  local expected_exit="$2"
  local expected_text="${3:-}"
  local fixture_dir="$FIXTURES_DIR/pass-session-auth-security"
  local temp_repo
  temp_repo="$(mktemp -d)"
  trap 'rm -rf "$temp_repo"' RETURN

  cp -R "$fixture_dir/." "$temp_repo/"
  cp "$ROOT_DIR/.gitattributes" "$temp_repo/.gitattributes"
  git -C "$temp_repo" init -q
  git -C "$temp_repo" -c user.name="$FIXTURE_GIT_AUTHOR_NAME" -c user.email="$FIXTURE_GIT_AUTHOR_EMAIL" commit --allow-empty -qm "base"
  "${PYTHON_BIN[@]}" - \
    "$temp_repo/services/api-rs/src/app/router.rs" \
    "$temp_repo/services/api-rs/src/models.rs" \
    "$temp_repo/services/api-rs/src/transport/http/handlers/dm.rs" \
    "$temp_repo/docs/contracts/runtime-rest.openapi.yaml" \
    "$mutation_name" <<'PY'
import pathlib
import sys

router_path = pathlib.Path(sys.argv[1])
models_path = pathlib.Path(sys.argv[2])
handler_path = pathlib.Path(sys.argv[3])
contract_path = pathlib.Path(sys.argv[4])
mutation_name = sys.argv[5]

router_text = router_path.read_text()
old_router = 'Router::new().route("/dm/threads", get(list_dm_threads));'
new_router = '''Router::new()
        .route("/dm/threads", get(list_dm_threads))
        .route("/dm/threads/:thread_id/read", post(mark_dm_thread_read));'''
if old_router not in router_text:
    raise SystemExit("fixture router mutation target not found")
router_path.write_text(router_text.replace(old_router, new_router, 1))

models_path.write_text(models_path.read_text() + '''

pub struct DmThreadMarkReadRequest {
    pub last_read_seq: u64,
}

pub struct DmThreadMarkReadResponse {
    pub thread_id: String,
    pub last_read_seq: u64,
    pub unread: u32,
}
''')

handler_text = handler_path.read_text()
handler_text = handler_text.replace(
    'use axum::{extract::{Query, State}, Json};',
    'use axum::{extract::{Path, Query, State}, http::StatusCode, Json};',
    1,
)
handler_text = handler_text.replace(
    'models::{DmThreadListQuery, DmThreadPage},',
    'models::{DmThreadListQuery, DmThreadMarkReadRequest, DmThreadMarkReadResponse, DmThreadPage},',
    1,
)
handler_text += '''

pub async fn mark_dm_thread_read(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Path(thread_id): Path<String>,
    Json(payload): Json<DmThreadMarkReadRequest>,
) -> ApiResult<Json<DmThreadMarkReadResponse>> {
    if payload.last_read_seq == u64::MAX {
        return Err(bad_request(
            "last_read_seq_invalid",
            "last_read_seq is outside the known thread sequence range",
        ));
    }
    if false {
        return Err((
            StatusCode::NOT_FOUND,
            Json(crate::shared::errors::ApiError {
                code: "thread_not_found",
                message: "thread was not found for this identity",
            }),
        ));
    }
    Ok(Json(DmThreadMarkReadResponse {
        thread_id,
        last_read_seq: payload.last_read_seq,
        unread: 0,
    }))
}
'''
handler_path.write_text(handler_text)

contract_text = contract_path.read_text()
route_insert = '''  /dm/threads/{thread_id}/read:
    post:
      security:
        - CookieAuth: []
        - BearerAuth: []
      parameters:
        - in: path
          name: thread_id
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/DmThreadMarkReadRequest'
      responses:
        '200':
          description: Updated read position
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DmThreadMarkReadResponse'
        '400':
          description: Invalid read position
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                lastReadSeqInvalid:
                  value:
                    code: last_read_seq_invalid
                    message: last_read_seq is outside the known thread sequence range
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          description: Thread not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                threadNotFound:
                  value:
                    code: thread_not_found
                    message: thread was not found for this identity
        '500':
          $ref: '#/components/responses/InternalServerError'
'''
if 'components:\n' not in contract_text:
    raise SystemExit("fixture contract components target not found")
contract_text = contract_text.replace('components:\n', route_insert + 'components:\n', 1)
schema_insert = '''    DmThreadMarkReadRequest:
      type: object
      required: [last_read_seq]
      properties:
        last_read_seq:
          type: integer
          minimum: 0
    DmThreadMarkReadResponse:
      type: object
      required: [thread_id, last_read_seq, unread]
      properties:
        thread_id:
          type: string
        last_read_seq:
          type: integer
          minimum: 0
        unread:
          type: integer
          minimum: 0
'''
if '    ApiError:\n' not in contract_text:
    raise SystemExit("fixture contract schema target not found")
contract_text = contract_text.replace('    ApiError:\n', schema_insert + '    ApiError:\n', 1)
contract_text = contract_text.replace(
    '            - storage_unavailable',
    '            - storage_unavailable\n            - last_read_seq_invalid\n            - thread_not_found',
    1,
)

if mutation_name == "fail-dm-mark-read-scalar-bounds":
    old = '''        last_read_seq:
          type: integer
          minimum: 0'''
    new = '''        last_read_seq:
          type: integer'''
    if old not in contract_text:
        raise SystemExit("fixture contract scalar-bound mutation target not found")
    contract_text = contract_text.replace(old, new, 1)
elif mutation_name == "fail-dm-mark-read-response-last-read-scalar-bound":
    old = '''        last_read_seq:
          type: integer
          minimum: 0'''
    new = '''        last_read_seq:
          type: integer'''
    if contract_text.count(old) < 2:
        raise SystemExit("fixture response last_read_seq scalar-bound mutation target not found")
    before, separator, after = contract_text.partition(old)
    contract_text = before + separator + after.replace(old, new, 1)
elif mutation_name == "fail-dm-mark-read-response-unread-scalar-bound":
    old = '''        unread:
          type: integer
          minimum: 0'''
    new = '''        unread:
          type: integer'''
    if old not in contract_text:
        raise SystemExit("fixture response unread scalar-bound mutation target not found")
    contract_text = contract_text.replace(old, new, 1)
elif mutation_name != "pass-dm-mark-read-scalar-bounds":
    raise SystemExit(f"unknown fixture mutation: {mutation_name}")

contract_path.write_text(contract_text)
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
run_dm_mark_read_schema_fixture pass-dm-mark-read-scalar-bounds 0
run_path_parameter_format_fixture pass-path-parameter-format 0
run_response_header_schema_type_fixture pass-response-header-schema-ref 0
run_fixture pass-request-body-component 0
run_fixture pass-request-schema-alias 0
run_fixture pass-response-schema-alias 0
run_fixture pass-session-auth-security 0
run_fixture pass-server-channel-example-status 0
run_fixture fail-cookie-actions 1 "issue:hexrelay_csrf"
run_api_error_schema_shape_fixture fail-api-error-schema-shape 1 '`ApiError` must require fields [code, message] but documents [code]'
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
run_query_parameter_pattern_fixture fail-query-parameter-pattern 1 'uses query parameter `identity_id` with pattern `^[A-Za-z0-9_-]{3,64}$` at runtime but documents `<none>`'
run_fixture fail-request-body-required 1 "requestBody is not marked required"
run_fixture fail-request-schema-ref-direct 1 "FriendRequestCreateRequest"
run_fixture fail-request-schema-ref-alias 1 "FriendRequestCreateRequest"
run_fixture fail-rest-schema-field-types 1 'uses request schema `AuthVerifyRequest` field `signature` as type `string` at runtime but documents `integer`'
run_fixture fail-rest-schema-array-item-ref 1 'returns schema `DmFanoutCatchUpResponse` field `items` array items as schema `DmFanoutCatchUpItem` at runtime but documents `FriendRequestPage`'
run_dm_mark_read_schema_fixture fail-dm-mark-read-scalar-bounds 1 'uses request schema `DmThreadMarkReadRequest` field `last_read_seq` minimum `0` at runtime but documents `<none>`'
run_dm_mark_read_schema_fixture fail-dm-mark-read-response-last-read-scalar-bound 1 'returns schema `DmThreadMarkReadResponse` field `last_read_seq` minimum `0` at runtime but documents `<none>`'
run_dm_mark_read_schema_fixture fail-dm-mark-read-response-unread-scalar-bound 1 'returns schema `DmThreadMarkReadResponse` field `unread` minimum `0` at runtime but documents `<none>`'
run_server_channel_request_schema_fixture fail-rest-schema-array-item-pattern 1 'uses request schema `ServerChannelMessageCreateRequest` field `mention_identity_ids` array items pattern `^[A-Za-z0-9_-]{3,64}$` at runtime but documents `<none>`'
run_fixture fail-rest-schema-date-time-format 1 'returns schema `AuthVerifyResponse` field `expires_at` format `date-time` at runtime but documents `<none>`'
run_fixture fail-rest-schema-nullable-field 1 'uses request schema `DmFanoutCatchUpRequest` field `cursor` nullable `true` at runtime but documents `false`'
run_fixture fail-rest-schema-scalar-bounds 1 'uses request schema `DmFanoutCatchUpRequest` field `limit` maximum `100` at runtime but documents `50`'
run_fixture fail-rest-schema-enum-domain 1 'returns schema `DmFanoutCatchUpResponse` field `status` enum [blocked, ready] at runtime but documents [ready]'
run_server_channel_request_schema_fixture fail-rest-schema-serde-default-required 1 'uses request schema `ServerChannelMessageCreateRequest` with required fields [content] at runtime but documents [content, mention_identity_ids]'
run_fixture fail-rest-schema-string-pattern 1 'uses request schema `AuthVerifyRequest` field `identity_id` pattern `^[A-Za-z0-9_-]{3,64}$` at runtime but documents `<none>`'
run_fixture fail-rest-schema-nested-item-field-type 1 'returns schema `DmFanoutCatchUpResponse` field `items` array items reference schema `DmFanoutCatchUpItem` field `ciphertext` as type `string` at runtime but documents `integer`'
run_fixture fail-rest-schema-required-fields 1 'uses request schema `AuthVerifyRequest` with required fields [challenge_id, identity_id, signature] at runtime but documents [challenge_id, identity_id]'
run_fixture fail-realtime-error-envelope-semantics 1 'Realtime runtime event `error` uses data fields [code, message] but documents [code]'
run_fixture fail-realtime-envelope-semantics 1 'Realtime runtime event `realtime.connected` uses data fields [state] but documents [status]'
run_fixture fail-realtime-signal-envelope-semantics 1 'Realtime runtime event `call.signal.offer` uses data fields [call_id, from_identity_id, sdp_offer, to_identity_id] but documents [call_id, from_identity_id, to_identity_id]'
run_fixture fail-realtime-signaling-semantics 1 'Realtime runtime event `call.signal.offer` requires from_identity_id/session-identity parity at runtime but does not require it'
run_fixture fail-response-header 1 'returns response header `Set-Cookie` for HTTP 200 at runtime but is missing it'
run_response_header_schema_type_fixture fail-response-header-schema-type 1 'returns response header `Set-Cookie` for HTTP 200 as type `string` at runtime but documents `integer`'
run_response_builder_success_schema_fixture fail-response-builder-success-schema 1 'POST /dev/testing/sessions returns response schema `TestingSessionCreateResponse` for HTTP 200 at runtime but documents `AuthVerifyResponse`'
run_fixture fail-response-schema-ref 1 "PresenceWatcherListResponse"
run_fixture fail-server-channel-example-status 1 "missing tracked HTTP 400 route-level error examples for ApiError codes [reply_target_invalid]"
run_fixture fail-session-auth-401 1 "missing a 401 response"
run_fixture fail-session-auth-security 1 "documents security schemes [CookieAuth] instead of [BearerAuth, CookieAuth]"
run_fixture fail-session-auth-500 1 "missing a 500 response"
run_fixture fail-success-content 1 "documents no success schema"
run_fixture fail-unexpected-request-body 1 "documents a requestBody but runtime handler has no request-body extractor"
run_fixture fail-missing-example 1 "thread_not_found"

printf '[contract-parity-test] Fixture regressions passed\n'
