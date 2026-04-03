#!/usr/bin/env bash
set -euo pipefail

base_sha="${1:-}"
head_sha="${2:-HEAD}"

if [ -z "${base_sha}" ] || [ "${base_sha}" = "0000000000000000000000000000000000000000" ]; then
  base_sha="$(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD origin/master 2>/dev/null || git rev-parse HEAD~1)"
fi

api_contract="docs/contracts/runtime-rest-v1.openapi.yaml"
realtime_contract="docs/contracts/realtime-events-runtime-v1.asyncapi.yaml"

if command -v py >/dev/null 2>&1; then
  PYTHON_BIN=(py -3)
elif command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN=(python3)
elif command -v python >/dev/null 2>&1; then
  PYTHON_BIN=(python)
else
  echo "::error::python3, python, or py -3 is required for contract parity validation."
  exit 1
fi

api_surface_files=(
  'services/api-rs/src/models.rs'
  'services/api-rs/src/app/router.rs'
  'services/api-rs/src/domain/**/*.rs'
  'services/api-rs/src/shared/errors.rs'
  'services/api-rs/src/transport/http/middleware/auth.rs'
  'services/api-rs/src/transport/http/middleware/authorization.rs'
  'services/api-rs/src/transport/http/middleware/rate_limit.rs'
  'services/api-rs/src/transport/http/handlers/*.rs'
)

api_surface_changes="$(git diff --name-only "${base_sha}" "${head_sha}" -- "${api_surface_files[@]}")"

realtime_surface_files=(
  'services/realtime-rs/src/app/router.rs'
  'services/realtime-rs/src/domain/events/*.rs'
  'services/realtime-rs/src/transport/ws/middleware/rate_limit.rs'
  'services/realtime-rs/src/transport/ws/handlers/*.rs'
)

realtime_surface_changes="$(git diff --name-only "${base_sha}" "${head_sha}" -- "${realtime_surface_files[@]}")"

api_contract_changed=0
if git diff --name-only "${base_sha}" "${head_sha}" -- "${api_contract}" | grep -qxF "${api_contract}"; then
  api_contract_changed=1
fi

realtime_contract_changed=0
if git diff --name-only "${base_sha}" "${head_sha}" -- "${realtime_contract}" | grep -qxF "${realtime_contract}"; then
  realtime_contract_changed=1
fi

extract_api_runtime_inventory() {
  "${PYTHON_BIN[@]}" - "$1" <<'PY'
import pathlib
import re
import sys

text = pathlib.Path(sys.argv[1]).read_text()
entries = set()
index = 0
needle = '.route('

while True:
    start = text.find(needle, index)
    if start == -1:
        break
    cursor = start + len(needle)
    depth = 1
    while cursor < len(text) and depth > 0:
        char = text[cursor]
        if char == '(':
            depth += 1
        elif char == ')':
            depth -= 1
        cursor += 1

    block = text[start:cursor]
    path_match = re.search(r'\.route\(\s*"([^"]+)"', block, re.S)
    if path_match:
        path = re.sub(r':([A-Za-z0-9_]+)', r'{\1}', path_match.group(1))
        for method in re.findall(r'\b(get|post|put|patch|delete)\s*\(', block):
            entries.add(f"{method.upper()} {path}")
    index = cursor

print("\n".join(sorted(entries)))
PY
}

extract_openapi_contract_inventory() {
  "${PYTHON_BIN[@]}" - "$1" <<'PY'
import pathlib
import re
import sys

lines = pathlib.Path(sys.argv[1]).read_text().splitlines()
entries = set()
in_paths = False
current_path = None

for line in lines:
    if not in_paths:
        if line.strip() == 'paths:':
            in_paths = True
        continue

    if re.match(r'^\S', line):
        break

    path_match = re.match(r'^  (/[^:]+):\s*$', line)
    if path_match:
        current_path = path_match.group(1)
        continue

    method_match = re.match(r'^    (get|post|put|patch|delete):\s*$', line)
    if method_match and current_path:
        entries.add(f"{method_match.group(1).upper()} {current_path}")

print("\n".join(sorted(entries)))
PY
}

extract_api_runtime_error_codes() {
  "${PYTHON_BIN[@]}" - "$@" <<'PY'
import pathlib
import re
import sys

codes = set()
for pattern in sys.argv[1:]:
    for path in pathlib.Path('.').glob(pattern):
        if path.is_file():
            text = path.read_text()
            codes.update(re.findall(r'\b(?:bad_request|unauthorized|forbidden|conflict|too_many_requests|internal_error)\(\s*"([^"]+)"', text, re.S))
            codes.update(re.findall(r'ApiError\s*\{\s*code:\s*"([^"]+)"', text, re.S))

print("\n".join(sorted(codes)))
PY
}

extract_openapi_contract_error_codes() {
  "${PYTHON_BIN[@]}" - "$1" <<'PY'
import pathlib
import re
import sys

lines = pathlib.Path(sys.argv[1]).read_text().splitlines()
codes = []
in_api_error = False
in_code_enum = False

for line in lines:
    if not in_api_error:
        if re.match(r'^\s{4}ApiError:\s*$', line):
            in_api_error = True
        continue

    if in_api_error and re.match(r'^\s{4}[A-Za-z].*:\s*$', line) and not re.match(r'^\s{4}ApiError:\s*$', line):
        break

    if not in_code_enum:
        if re.match(r'^\s{8}code:\s*$', line):
            continue
        if re.match(r'^\s{10}enum:\s*$', line):
            in_code_enum = True
        continue

    match = re.match(r'^\s{12}-\s+([^\s]+)\s*$', line)
    if match:
        codes.append(match.group(1))
        continue

    if in_code_enum and not re.match(r'^\s{12}-\s+', line):
        break

print("\n".join(sorted(set(codes))))
PY
}

validate_api_semantic_contracts() {
  "${PYTHON_BIN[@]}" - "$1" <<'PY'
import pathlib
import re
import sys


TRACKED_ERROR_STATUS_TOKENS = {
    '400': ('bad_request(',),
    '403': ('forbidden(',),
    '404': ('StatusCode::NOT_FOUND',),
    '409': ('conflict(',),
    '429': ('too_many_requests(',),
}
TRACKED_ERROR_STATUS_PATTERN = '|'.join(sorted(TRACKED_ERROR_STATUS_TOKENS))


def route_blocks(text: str):
    needle = '.route('
    index = 0
    while True:
        start = text.find(needle, index)
        if start == -1:
            return
        cursor = start + len(needle)
        depth = 1
        while cursor < len(text) and depth > 0:
            char = text[cursor]
            if char == '(':
                depth += 1
            elif char == ')':
                depth -= 1
            cursor += 1
        yield text[start:cursor]
        index = cursor


def extract_function_blocks(paths):
    functions = {}
    pattern = re.compile(r'(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\((.*?)\)\s*->', re.S)
    for path in paths:
        text = pathlib.Path(path).read_text()
        for match in pattern.finditer(text):
            name = match.group(1)
            params = match.group(2)
            body_start = text.find('{', match.end())
            if body_start == -1:
                continue
            return_type = text[match.end():body_start].strip()
            cursor = body_start + 1
            depth = 1
            while cursor < len(text) and depth > 0:
                char = text[cursor]
                if char == '{':
                    depth += 1
                elif char == '}':
                    depth -= 1
                cursor += 1
            body = text[body_start:cursor]
            existing = functions.get(name)
            functions[name] = {
                'has_auth': 'AuthSession' in params if existing is None else existing['has_auth'],
                'has_csrf': 'enforce_csrf_for_cookie_auth(' in body if existing is None else existing['has_csrf'],
                'has_json_body': 'Json<' in params if existing is None else existing['has_json_body'],
                'has_path_params': 'Path<' in params if existing is None else existing['has_path_params'],
                'query_type': extract_query_type(params) if existing is None else existing['query_type'],
                'error_statuses': set(),
                'return_type': return_type,
                'body': body,
            }
    for name in list(functions):
        functions[name]['error_statuses'] = infer_error_statuses(name, functions)
    return functions


def extract_query_type(params: str):
    match = re.search(r'(?:^|[^A-Za-z0-9_:])(?:axum::extract::)?Query\s*\(?.*?:\s*(?:axum::extract::)?Query<\s*([A-Za-z0-9_]+)\s*>', params, re.S)
    if match:
        return match.group(1)
    return None


def extract_query_struct_fields(models_path: pathlib.Path):
    text = models_path.read_text()
    structs = {}
    struct_pattern = re.compile(r'pub struct\s+(\w+)\s*\{(.*?)\n\}', re.S)
    field_pattern = re.compile(r'pub\s+(\w+):')

    for match in struct_pattern.finditer(text):
        name = match.group(1)
        body = match.group(2)
        if not name.endswith('Query'):
            continue
        structs[name] = sorted(set(field_pattern.findall(body)))

    return structs


def infer_error_statuses(handler_name, functions, stack=None, follow_helpers=True):
    if stack is None:
        stack = set()
    if handler_name in stack:
        return set()

    function = functions.get(handler_name)
    if not function:
        return set()

    body = function.get('body', '')
    statuses = set()

    for status, tokens in TRACKED_ERROR_STATUS_TOKENS.items():
        if any(token in body for token in tokens):
            statuses.add(status)

    if not follow_helpers:
        return statuses

    helper_names = set(re.findall(r'\b(map_[A-Za-z0-9_]+)\b', body))
    delegate_calls = set(re.findall(r'\b(\w+)\s*\([^;]*\)\.await', body))
    for callee_name in sorted(helper_names | delegate_calls):
        if callee_name == handler_name:
            continue
        statuses.update(
            infer_error_statuses(
                callee_name, functions, stack | {handler_name}, follow_helpers=follow_helpers
            )
        )

    return statuses


def infer_success_status(handler_name, functions, stack=None):
    if stack is None:
        stack = set()
    if handler_name in stack:
        return None

    function = functions.get(handler_name)
    if not function:
        return None

    body = function.get('body', '')
    if 'StatusCode::CREATED' in body:
        return '201'
    if 'StatusCode::NO_CONTENT' in body:
        return '204'
    if 'StatusCode::ACCEPTED' in body:
        return '202'

    body_inner = body.strip()
    if body_inner.startswith('{') and body_inner.endswith('}'):
        body_inner = body_inner[1:-1].strip()
    delegate_match = re.fullmatch(r'(?:return\s+)?(\w+)\([^;]*\)\.await;?', body_inner, re.S)
    if delegate_match:
        delegated_status = infer_success_status(
            delegate_match.group(1), functions, stack | {handler_name}
        )
        if delegated_status:
            return delegated_status

    if 'Json<' in function.get('return_type', ''):
        return '200'

    return None


def extract_runtime_semantics(router_text: str, function_semantics, query_struct_fields):
    semantics = {}
    for block in route_blocks(router_text):
        path_match = re.search(r'\.route\(\s*"([^"]+)"', block, re.S)
        if not path_match:
            continue
        path = re.sub(r':([A-Za-z0-9_]+)', r'{\1}', path_match.group(1))
        path_param_names = sorted(set(re.findall(r'\{([A-Za-z0-9_]+)\}', path)))
        for method, handler in re.findall(r'\b(get|post|put|patch|delete)\s*\(\s*(\w+)\s*\)', block):
            handler_semantics = function_semantics.get(handler, {})
            query_type = handler_semantics.get('query_type')
            semantics[(method.upper(), path)] = {
                'handler': handler,
                'has_auth': bool(handler_semantics.get('has_auth')),
                'has_csrf': bool(handler_semantics.get('has_csrf')),
                'has_json_body': bool(handler_semantics.get('has_json_body')),
                'path_param_names': path_param_names if handler_semantics.get('has_path_params') else [],
                'query_param_names': query_struct_fields.get(query_type, []) if query_type else [],
                'error_statuses': infer_error_statuses(
                    handler,
                    function_semantics,
                    follow_helpers=method.upper() != 'GET',
                ),
                'success_status': infer_success_status(handler, function_semantics),
            }
    return semantics


def extract_contract_semantics(contract_path: pathlib.Path):
    lines = contract_path.read_text().splitlines()
    semantics = {}
    in_paths = False
    current_path = None
    current_method = None
    current_parameter_in = None

    for line in lines:
        if not in_paths:
            if line.strip() == 'paths:':
                in_paths = True
            continue

        if re.match(r'^\S', line):
            break

        path_match = re.match(r'^  (/[^:]+):\s*$', line)
        if path_match:
            current_path = path_match.group(1)
            current_method = None
            current_parameter_in = None
            continue

        method_match = re.match(r'^    (get|post|put|patch|delete):\s*$', line)
        if method_match and current_path:
            current_method = method_match.group(1).upper()
            current_parameter_in = None
            semantics[(current_method, current_path)] = {
                'has_security': False,
                'has_401': False,
                'has_500': False,
                'has_csrf': False,
                'has_request_body': False,
                'path_parameters': set(),
                'query_parameters': set(),
                'error_responses': set(),
                'success_responses': set(),
            }
            continue

        if not current_path or not current_method:
            continue

        if re.match(r'^      security:\s*$', line):
            semantics[(current_method, current_path)]['has_security'] = True
        elif re.match(r'^      requestBody:\s*$', line):
            semantics[(current_method, current_path)]['has_request_body'] = True
        elif "#/components/parameters/CsrfTokenHeader" in line:
            semantics[(current_method, current_path)]['has_csrf'] = True
        elif re.match(r"^        '401':\s*$", line):
            semantics[(current_method, current_path)]['has_401'] = True
        elif re.match(r"^        '500':\s*$", line):
            semantics[(current_method, current_path)]['has_500'] = True
        else:
            if re.match(r'^      [A-Za-z_][A-Za-z0-9_]*:\s*$', line):
                current_parameter_in = None
            parameter_match = re.match(r'^        - in: (path|query)\s*$', line)
            if parameter_match:
                current_parameter_in = parameter_match.group(1)
                continue
            other_parameter_match = re.match(r'^        - in: [A-Za-z_][A-Za-z0-9_]*\s*$', line)
            if other_parameter_match:
                current_parameter_in = None
                continue
            parameter_name_match = re.match(r'^          name: ([A-Za-z0-9_]+)\s*$', line)
            if parameter_name_match and current_parameter_in in {'path', 'query'}:
                if current_parameter_in == 'path':
                    semantics[(current_method, current_path)]['path_parameters'].add(parameter_name_match.group(1))
                else:
                    semantics[(current_method, current_path)]['query_parameters'].add(parameter_name_match.group(1))
                continue
            success_match = re.match(r"^        '(2\d\d)':\s*$", line)
            if success_match:
                semantics[(current_method, current_path)]['success_responses'].add(
                    success_match.group(1)
                )
                continue
            error_match = re.match(
                rf"^        '(({TRACKED_ERROR_STATUS_PATTERN}))':\s*$", line
            )
            if error_match:
                semantics[(current_method, current_path)]['error_responses'].add(
                    error_match.group(1)
                )

    return semantics


contract_path = pathlib.Path(sys.argv[1])
router_text = pathlib.Path('services/api-rs/src/app/router.rs').read_text()
handler_paths = sorted(pathlib.Path('services/api-rs/src/transport/http/handlers').glob('*.rs'))
query_struct_fields = extract_query_struct_fields(pathlib.Path('services/api-rs/src/models.rs'))

function_semantics = extract_function_blocks(handler_paths)
runtime_semantics = extract_runtime_semantics(router_text, function_semantics, query_struct_fields)
contract_semantics = extract_contract_semantics(contract_path)

errors = []

for key, runtime in sorted(runtime_semantics.items()):
    method, path = key
    contract = contract_semantics.get(key)
    if contract is None:
        continue
    if runtime['has_auth'] and not contract['has_security']:
        errors.append(f"::error::{method} {path} uses AuthSession at runtime but is missing security requirements in {contract_path}.")
    if runtime['has_auth'] and not contract['has_401']:
        errors.append(f"::error::{method} {path} uses AuthSession at runtime but is missing a 401 response in {contract_path}.")
    if runtime['has_auth'] and not contract['has_500']:
        errors.append(f"::error::{method} {path} uses AuthSession-backed runtime auth/storage but is missing a 500 response in {contract_path}.")
    if runtime['has_csrf'] and not contract['has_csrf']:
        errors.append(f"::error::{method} {path} enforces CSRF at runtime but is missing the CsrfTokenHeader parameter in {contract_path}.")
    if runtime['has_json_body'] and not contract['has_request_body']:
        errors.append(f"::error::{method} {path} accepts a Json request body at runtime but is missing requestBody in {contract_path}.")
    missing_path_parameters = sorted(set(runtime['path_param_names']) - contract['path_parameters'])
    for parameter_name in missing_path_parameters:
        errors.append(f"::error::{method} {path} uses path parameter `{parameter_name}` at runtime but is missing an `in: path` parameter in {contract_path}.")
    missing_query_parameters = sorted(set(runtime['query_param_names']) - contract['query_parameters'])
    for parameter_name in missing_query_parameters:
        errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` at runtime but is missing an `in: query` parameter in {contract_path}.")
    missing_error_responses = sorted(runtime['error_statuses'] - contract['error_responses'])
    for status_code in missing_error_responses:
        errors.append(f"::error::{method} {path} can return HTTP {status_code} at runtime but is missing that error response in {contract_path}.")
    if runtime['success_status'] and runtime['success_status'] not in contract['success_responses']:
        errors.append(f"::error::{method} {path} returns HTTP {runtime['success_status']} at runtime but is missing that success response in {contract_path}.")

if errors:
    print("\n".join(errors))
    sys.exit(1)
PY
}

extract_realtime_runtime_events() {
  "${PYTHON_BIN[@]}" - "$1" <<'PY'
import pathlib
import re
import sys

text = pathlib.Path(sys.argv[1]).read_text()
events = set(re.findall(r'"(call\.signal\.[^"]+)"\s*=>', text))
events.update(re.findall(r'event_type:\s*"([^"]+)"\.to_string\(\)', text))
print("\n".join(sorted(events)))
PY
}

extract_asyncapi_contract_events() {
  "${PYTHON_BIN[@]}" - "$1" <<'PY'
import pathlib
import re
import sys

lines = pathlib.Path(sys.argv[1]).read_text().splitlines()
events = set()
in_channels = False

for line in lines:
    if not in_channels:
        if line.strip() == 'channels:':
            in_channels = True
        continue

    if re.match(r'^operations:', line):
        break

    match = re.match(r'^    address:\s*([^\s]+)\s*$', line)
    if match:
        events.add(match.group(1))

print("\n".join(sorted(events)))
PY
}

extract_realtime_runtime_error_codes() {
  "${PYTHON_BIN[@]}" - "$@" <<'PY'
import pathlib
import re
import sys

codes = set()
for path in sys.argv[1:]:
    text = pathlib.Path(path).read_text()
    codes.update(re.findall(r'build_error_event\(\s*"([^"]+)"', text, re.S))
    codes.update(re.findall(r'ws_rejection\(\s*[^,]+,\s*"([^"]+)"', text, re.S))

print("\n".join(sorted(codes)))
PY
}

extract_asyncapi_contract_error_codes() {
  "${PYTHON_BIN[@]}" - "$1" <<'PY'
import pathlib
import re
import sys

lines = pathlib.Path(sys.argv[1]).read_text().splitlines()
codes = []
in_error_schema = False
in_code_enum = False

for line in lines:
    if not in_error_schema:
        if re.match(r'^\s{4}ErrorDataV1:\s*$', line):
            in_error_schema = True
        continue

    if in_error_schema and re.match(r'^\s{4}[A-Za-z].*:\s*$', line) and not re.match(r'^\s{4}ErrorDataV1:\s*$', line):
        break

    if not in_code_enum:
        if re.match(r'^\s{8}code:\s*$', line):
            continue
        if re.match(r'^\s{10}enum:\s*$', line):
            in_code_enum = True
        continue

    match = re.match(r'^\s{12}-\s+([^\s]+)\s*$', line)
    if match:
        codes.append(match.group(1))
        continue

    if in_code_enum and not re.match(r'^\s{12}-\s+', line):
        break

print("\n".join(sorted(set(codes))))
PY
}

compare_inventory() {
  local label="$1"
  local runtime_inventory="$2"
  local contract_inventory="$3"
  local runtime_file
  local contract_file
  runtime_file="$(mktemp)"
  contract_file="$(mktemp)"

  printf '%s\n' "${runtime_inventory}" | sed '/^$/d' | sort -u > "${runtime_file}"
  printf '%s\n' "${contract_inventory}" | sed '/^$/d' | sort -u > "${contract_file}"

  local missing_from_contract
  local extra_in_contract
  missing_from_contract="$(comm -23 "${runtime_file}" "${contract_file}" || true)"
  extra_in_contract="$(comm -13 "${runtime_file}" "${contract_file}" || true)"

  if [ -n "${missing_from_contract}" ] || [ -n "${extra_in_contract}" ]; then
    echo "::error::${label} drift detected between runtime inventory and contract."
    if [ -n "${missing_from_contract}" ]; then
      echo "Missing from contract:"
      echo "${missing_from_contract}"
    fi
    if [ -n "${extra_in_contract}" ]; then
      echo "Only in contract:"
      echo "${extra_in_contract}"
    fi
    errors=1
  fi

  rm -f "${runtime_file}" "${contract_file}"
}

api_runtime_inventory="$(extract_api_runtime_inventory 'services/api-rs/src/app/router.rs')"
api_contract_inventory="$(extract_openapi_contract_inventory "${api_contract}")"
api_runtime_error_codes="$(extract_api_runtime_error_codes \
  'services/api-rs/src/domain/**/*.rs' \
  'services/api-rs/src/shared/errors.rs' \
  'services/api-rs/src/transport/http/middleware/auth.rs' \
  'services/api-rs/src/transport/http/middleware/authorization.rs' \
  'services/api-rs/src/transport/http/middleware/rate_limit.rs' \
  'services/api-rs/src/transport/http/handlers/*.rs')"
api_contract_error_codes="$(extract_openapi_contract_error_codes "${api_contract}")"
realtime_runtime_events="$(extract_realtime_runtime_events 'services/realtime-rs/src/domain/events/service.rs')"
realtime_contract_events="$(extract_asyncapi_contract_events "${realtime_contract}")"
realtime_runtime_error_codes="$(extract_realtime_runtime_error_codes 'services/realtime-rs/src/domain/events/service.rs' 'services/realtime-rs/src/transport/ws/handlers/gateway.rs')"
realtime_contract_error_codes="$(extract_asyncapi_contract_error_codes "${realtime_contract}")"

errors=0

if [ -n "${api_surface_changes}" ] && [ "${api_contract_changed}" -ne 1 ]; then
  echo "::error::API contract-surface files changed but ${api_contract} was not updated."
  echo "Changed API surface files:"
  echo "${api_surface_changes}"
  errors=1
fi

if [ -n "${realtime_surface_changes}" ] && [ "${realtime_contract_changed}" -ne 1 ]; then
  echo "::error::Realtime websocket/event surface changed but ${realtime_contract} was not updated."
  echo "Changed realtime surface files:"
  echo "${realtime_surface_changes}"
  errors=1
fi

if ! grep -q '^openapi:' "${api_contract}"; then
  echo "::error::${api_contract} is missing required openapi version field."
  errors=1
fi

if ! grep -q '^asyncapi:' "${realtime_contract}"; then
  echo "::error::${realtime_contract} is missing required asyncapi version field."
  errors=1
fi

compare_inventory "API route inventory" "${api_runtime_inventory}" "${api_contract_inventory}"
compare_inventory "API error-code inventory" "${api_runtime_error_codes}" "${api_contract_error_codes}"
compare_inventory "Realtime event inventory" "${realtime_runtime_events}" "${realtime_contract_events}"
compare_inventory "Realtime error-code inventory" "${realtime_runtime_error_codes}" "${realtime_contract_error_codes}"

if ! validate_api_semantic_contracts "${api_contract}"; then
  errors=1
fi

if [ "${errors}" -ne 0 ]; then
  echo "[contract-parity] Update runtime contract docs when API/realtime surface changes."
  exit 1
fi

echo "[contract-parity] Runtime contract parity checks passed"
