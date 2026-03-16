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
            codes.update(re.findall(r'\b(?:bad_request|unauthorized|conflict|too_many_requests|internal_error)\(\s*"([^"]+)"', text, re.S))
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

if [ "${errors}" -ne 0 ]; then
  echo "[contract-parity] Update runtime contract docs when API/realtime surface changes."
  exit 1
fi

echo "[contract-parity] Runtime contract parity checks passed"
