from __future__ import annotations

import pathlib
import re

try:
    from .api_contract import extract_api_error_schema_shape, extract_contract_semantics
    from .api_runtime import (
        extract_function_blocks,
        extract_openapi_tracked_schema_fields,
        extract_query_struct_fields,
        extract_runtime_semantics,
        extract_tracked_schema_fields,
    )
    from .api_rules import *  # noqa: F403
except ImportError:  # pragma: no cover
    from api_contract import extract_api_error_schema_shape, extract_contract_semantics  # type: ignore
    from api_runtime import (  # type: ignore
        extract_function_blocks,
        extract_openapi_tracked_schema_fields,
        extract_query_struct_fields,
        extract_runtime_semantics,
        extract_tracked_schema_fields,
    )
    from api_rules import *  # type: ignore  # noqa: F403


def extract_api_runtime_inventory(path: str) -> str:
    text = pathlib.Path(path).read_text()
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

    return "\n".join(sorted(entries))

def extract_openapi_contract_inventory(path: str) -> str:
    lines = pathlib.Path(path).read_text().splitlines()
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

    return "\n".join(sorted(entries))

def extract_api_runtime_error_codes(*patterns: str) -> str:
    codes = set()
    for pattern in patterns:
        for path in pathlib.Path('.').glob(pattern):
            if path.is_file():
                text = path.read_text()
                codes.update(re.findall(r'\b(?:bad_request|unauthorized|forbidden|conflict|too_many_requests|internal_error)\(\s*"([^"]+)"', text, re.S))
                codes.update(re.findall(r'ApiError\s*\{\s*code:\s*"([^"]+)"', text, re.S))

    return "\n".join(sorted(codes))

def extract_openapi_contract_error_codes(path: str) -> str:
    lines = pathlib.Path(path).read_text().splitlines()
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

    return "\n".join(sorted(set(codes)))

def validate_api_semantic_contracts(contract_path_str: str) -> int:
    contract_path = pathlib.Path(contract_path_str)
    contract_lines = contract_path.read_text().splitlines()
    router_text = pathlib.Path('services/api-rs/src/app/router.rs').read_text()
    handler_paths = sorted(pathlib.Path('services/api-rs/src/transport/http/handlers').glob('*.rs'))
    models_path = pathlib.Path('services/api-rs/src/models.rs')
    query_struct_fields = extract_query_struct_fields(models_path)
    runtime_schema_fields = extract_tracked_schema_fields(models_path)
    contract_schema_fields = extract_openapi_tracked_schema_fields(contract_path)
    api_error_schema_shape = extract_api_error_schema_shape(contract_lines)

    function_semantics, route_handler_lookup, local_lookup = extract_function_blocks(handler_paths)
    runtime_semantics = extract_runtime_semantics(
        router_text, function_semantics, route_handler_lookup, local_lookup, query_struct_fields
    )
    contract_semantics = extract_contract_semantics(contract_path)

    errors = []

    def format_bool(value):
        return 'true' if value else 'false'

    requires_api_error_schema = api_error_schema_shape['present'] or any(
        runtime['error_statuses'] & TRACKED_ERROR_SCHEMA_STATUSES
        for runtime in runtime_semantics.values()
    )
    if requires_api_error_schema and not api_error_schema_shape['present']:
        errors.append(f"::error::{contract_path} is missing the shared `{API_ERROR_SCHEMA_NAME}` schema.")
    elif api_error_schema_shape['present']:
        documented_schema_type = api_error_schema_shape.get('schema_type')
        if documented_schema_type != 'object':
            actual = documented_schema_type or '<none>'
            errors.append(
                f"::error::{contract_path} documents `{API_ERROR_SCHEMA_NAME}` as type `{actual}` instead of `object`."
            )

        documented_required = api_error_schema_shape.get('required', set())
        if documented_required != API_ERROR_REQUIRED_FIELDS:
            expected = ', '.join(sorted(API_ERROR_REQUIRED_FIELDS))
            documented = ', '.join(sorted(documented_required)) or '<none>'
            errors.append(
                f"::error::{contract_path} `{API_ERROR_SCHEMA_NAME}` must require fields [{expected}] but documents [{documented}]."
            )

        documented_field_types = api_error_schema_shape.get('field_types', {})
        for field_name, expected_type in sorted(API_ERROR_FIELD_TYPES.items()):
            documented_type = documented_field_types.get(field_name)
            if documented_type != expected_type:
                actual = documented_type or '<none>'
                errors.append(
                    f"::error::{contract_path} `{API_ERROR_SCHEMA_NAME}` field `{field_name}` must be type `{expected_type}` but documents `{actual}`."
                )

    def compare_tracked_rest_schema(schema_name, relation, method, path, seen=None):
        if schema_name not in runtime_schema_fields or schema_name not in contract_schema_fields:
            return
        if seen is None:
            seen = set()
        if schema_name in seen:
            return
        seen = seen | {schema_name}

        runtime_fields = runtime_schema_fields[schema_name]
        documented_fields = contract_schema_fields[schema_name]
        runtime_required = {name for name, field in runtime_fields.items() if field['required']}
        documented_required = {name for name, field in documented_fields.items() if field['required']}
        if runtime_required != documented_required:
            documented = ', '.join(sorted(documented_required)) or '<none>'
            expected = ', '.join(sorted(runtime_required)) or '<none>'
            errors.append(
                f"::error::{method} {path} {relation} `{schema_name}` with required fields [{expected}] at runtime but documents [{documented}] in {contract_path}."
            )

        for field_name, runtime_field in sorted(runtime_fields.items()):
            documented_field = documented_fields.get(field_name, {})
            runtime_type = runtime_field.get('schema_type')
            documented_type = documented_field.get('schema_type')
            if runtime_type != documented_type:
                actual_type = documented_type or '<none>'
                errors.append(
                    f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` as type `{runtime_type}` at runtime but documents `{actual_type}` in {contract_path}."
                )
                continue

            runtime_nullable = bool(runtime_field.get('nullable', False))
            documented_nullable = bool(documented_field.get('nullable', False))
            if runtime_nullable != documented_nullable:
                errors.append(
                    f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` nullable `{format_bool(runtime_nullable)}` at runtime but documents `{format_bool(documented_nullable)}` in {contract_path}."
                )
                continue

            runtime_format = runtime_field.get('format')
            if runtime_format:
                documented_format = documented_field.get('format')
                if runtime_format != documented_format:
                    actual_format = documented_format or '<none>'
                    errors.append(
                        f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` format `{runtime_format}` at runtime but documents `{actual_format}` in {contract_path}."
                    )
                    continue

            runtime_pattern = runtime_field.get('pattern')
            if runtime_pattern:
                documented_pattern = documented_field.get('pattern')
                if runtime_pattern != documented_pattern:
                    actual_pattern = documented_pattern or '<none>'
                    errors.append(
                        f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` pattern `{runtime_pattern}` at runtime but documents `{actual_pattern}` in {contract_path}."
                    )
                    continue

            runtime_enum = set(runtime_field.get('enum', ()))
            if runtime_enum:
                documented_enum = set(documented_field.get('enum', set()))
                if runtime_enum != documented_enum:
                    expected = ', '.join(sorted(runtime_enum))
                    documented = ', '.join(sorted(documented_enum)) or '<none>'
                    errors.append(
                        f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` enum [{expected}] at runtime but documents [{documented}] in {contract_path}."
                    )
                    continue

            constraint_mismatch = False
            for constraint_key, label in REST_SCHEMA_CONSTRAINT_LABELS:
                runtime_constraint = runtime_field.get(constraint_key)
                if runtime_constraint is None:
                    continue
                documented_constraint = documented_field.get(constraint_key)
                if runtime_constraint != documented_constraint:
                    actual_constraint = documented_constraint if documented_constraint is not None else '<none>'
                    errors.append(
                        f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` {label} `{runtime_constraint}` at runtime but documents `{actual_constraint}` in {contract_path}."
                    )
                    constraint_mismatch = True
                    break
            if constraint_mismatch:
                continue

            runtime_ref = runtime_field.get('schema_ref')
            documented_ref = documented_field.get('schema_ref')
            if runtime_ref and runtime_ref != documented_ref:
                actual_ref = documented_ref or '<none>'
                errors.append(
                    f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` as schema `{runtime_ref}` at runtime but documents `{actual_ref}` in {contract_path}."
                )
                continue
            if runtime_ref:
                compare_tracked_rest_schema(
                    runtime_ref,
                    f"{relation} `{schema_name}` field `{field_name}` references schema",
                    method,
                    path,
                    seen,
                )

            runtime_item_type = runtime_field.get('item_schema_type')
            documented_item_type = documented_field.get('item_schema_type')
            if runtime_item_type and runtime_item_type != documented_item_type:
                actual_item_type = documented_item_type or '<none>'
                errors.append(
                    f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` array items as type `{runtime_item_type}` at runtime but documents `{actual_item_type}` in {contract_path}."
                )
                continue

            runtime_item_ref = runtime_field.get('item_schema_ref')
            documented_item_ref = documented_field.get('item_schema_ref')
            if runtime_item_ref and runtime_item_ref != documented_item_ref:
                actual_item_ref = documented_item_ref or '<none>'
                errors.append(
                    f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` array items as schema `{runtime_item_ref}` at runtime but documents `{actual_item_ref}` in {contract_path}."
                )
                continue
            if runtime_item_ref:
                compare_tracked_rest_schema(
                    runtime_item_ref,
                    f"{relation} `{schema_name}` field `{field_name}` array items reference schema",
                    method,
                    path,
                    seen,
                )

            runtime_item_pattern = runtime_field.get('item_pattern')
            if runtime_item_pattern:
                documented_item_pattern = documented_field.get('item_pattern')
                if runtime_item_pattern != documented_item_pattern:
                    actual_item_pattern = documented_item_pattern or '<none>'
                    errors.append(
                        f"::error::{method} {path} {relation} `{schema_name}` field `{field_name}` array items pattern `{runtime_item_pattern}` at runtime but documents `{actual_item_pattern}` in {contract_path}."
                    )
                    continue

    for key, runtime in sorted(runtime_semantics.items()):
        method, path = key
        contract = contract_semantics.get(key)
        if contract is None:
            continue
        if runtime['has_auth'] and contract['security_schemes'] != AUTH_SESSION_SECURITY_SCHEMES:
            documented = ', '.join(sorted(contract['security_schemes'])) or '<none>'
            expected = ', '.join(sorted(AUTH_SESSION_SECURITY_SCHEMES))
            errors.append(f"::error::{method} {path} requires session auth at runtime (AuthSession or server-membership authorizer extractor) but documents security schemes [{documented}] instead of [{expected}] in {contract_path}.")
        if runtime['has_internal_auth']:
            missing_internal_headers = sorted(INTERNAL_TOKEN_REQUIRED_HEADERS - contract['request_headers'])
            for header_name in missing_internal_headers:
                errors.append(f"::error::{method} {path} requires internal-token header `{header_name}` at runtime but is missing it from {contract_path}.")
            if contract['security_schemes']:
                documented = ', '.join(sorted(contract['security_schemes']))
                errors.append(f"::error::{method} {path} uses internal-token auth at runtime and should not declare session security schemes [{documented}] in {contract_path}.")
        if not runtime['has_auth'] and not runtime['has_internal_auth'] and contract['security_schemes']:
            documented = ', '.join(sorted(contract['security_schemes']))
            errors.append(f"::error::{method} {path} documents security schemes [{documented}] but runtime does not require session or internal-token auth in {contract_path}.")
        if runtime['has_401'] and not contract['has_401']:
            if runtime['has_auth']:
                errors.append(f"::error::{method} {path} requires session auth at runtime (AuthSession or server-membership authorizer extractor) but is missing a 401 response in {contract_path}.")
            elif runtime['has_internal_auth']:
                errors.append(f"::error::{method} {path} requires internal-token auth at runtime but is missing a 401 response in {contract_path}.")
            else:
                errors.append(f"::error::{method} {path} can return HTTP 401 at runtime via direct unauthorized emitters or local failure helpers but is missing a 401 response in {contract_path}.")
        if runtime['has_500'] and not contract['has_500']:
            if runtime['has_auth']:
                errors.append(f"::error::{method} {path} requires session-auth-backed runtime auth/storage (AuthSession or server-membership authorizer extractor) but is missing a 500 response in {contract_path}.")
            else:
                errors.append(f"::error::{method} {path} can return HTTP 500 at runtime via direct internal_error emitters or local helper/delegate flows but is missing a 500 response in {contract_path}.")
        if runtime['has_csrf'] and not contract['has_csrf']:
            errors.append(f"::error::{method} {path} enforces CSRF at runtime but is missing the CsrfTokenHeader parameter in {contract_path}.")
        if runtime['has_csrf'] and contract['has_csrf']:
            csrf_header = contract.get('csrf_header') or {}
            documented_name = csrf_header.get('name') or '<none>'
            documented_location = csrf_header.get('in') or '<none>'
            if documented_name != CSRF_HEADER_NAME or documented_location != 'header':
                errors.append(
                    f"::error::{method} {path} enforces CSRF header `{CSRF_HEADER_NAME}` at runtime but CsrfTokenHeader documents `{documented_name}` in `{documented_location}` in {contract_path}."
                )
            if csrf_header.get('required'):
                errors.append(
                    f"::error::{method} {path} enforces CSRF only for cookie-authenticated mutation requests at runtime but CsrfTokenHeader is marked always required in {contract_path}."
                )
            documented_type = csrf_header.get('schema_type')
            if documented_type != CSRF_HEADER_SCHEMA_TYPE:
                actual_type = documented_type or '<none>'
                errors.append(
                    f"::error::{method} {path} enforces CSRF header `{CSRF_HEADER_NAME}` as type `{CSRF_HEADER_SCHEMA_TYPE}` at runtime but documents `{actual_type}` in {contract_path}."
                )
        if not runtime['has_csrf'] and contract['has_csrf']:
            errors.append(
                f"::error::{method} {path} documents CsrfTokenHeader but runtime does not enforce CSRF in {contract_path}."
            )
        if runtime['has_json_body'] and not contract['has_request_body']:
            errors.append(f"::error::{method} {path} accepts a Json request body at runtime but is missing requestBody in {contract_path}.")
        if runtime['has_json_body'] and contract['has_request_body'] and not contract['request_body_required']:
            errors.append(f"::error::{method} {path} accepts a required Json request body at runtime but requestBody is not marked required in {contract_path}.")
        if runtime['has_json_body'] and contract['has_request_body']:
            documented_media_types = set(contract.get('request_body_media_types', set()))
            if documented_media_types != {JSON_REQUEST_MEDIA_TYPE}:
                documented = ', '.join(sorted(documented_media_types)) or '<none>'
                errors.append(
                    f"::error::{method} {path} accepts JSON request bodies at runtime but documents request media types [{documented}] instead of [{JSON_REQUEST_MEDIA_TYPE}] in {contract_path}."
                )
        if not runtime['has_request_body_extractor'] and contract['has_request_body']:
            errors.append(f"::error::{method} {path} documents a requestBody but runtime handler has no request-body extractor in {contract_path}.")
        if runtime['request_body_schema'] and contract['request_body_schema'] != runtime['request_body_schema']:
            documented = contract['request_body_schema'] or '<none>'
            errors.append(f"::error::{method} {path} accepts request body schema `{runtime['request_body_schema']}` at runtime but documents `{documented}` in {contract_path}.")
        compare_tracked_rest_schema(
            runtime['request_body_schema'],
            'uses request schema',
            method,
            path,
        )
        if runtime['response_body_schema'] and runtime['success_status']:
            documented = contract['response_schemas'].get(runtime['success_status'])
            if documented != runtime['response_body_schema']:
                actual = documented or '<none>'
                errors.append(f"::error::{method} {path} returns response schema `{runtime['response_body_schema']}` for HTTP {runtime['success_status']} at runtime but documents `{actual}` in {contract_path}.")
            else:
                compare_tracked_rest_schema(
                    runtime['response_body_schema'],
                    'returns schema',
                    method,
                    path,
                )
        if runtime['success_status'] and runtime['success_body_kind'] == 'none':
            documented = contract['response_schemas'].get(runtime['success_status'])
            if documented is not None:
                errors.append(f"::error::{method} {path} returns HTTP {runtime['success_status']} without a JSON success body at runtime but documents schema `{documented}` in {contract_path}.")
        if runtime['success_status'] and runtime['success_body_kind'] == 'json':
            documented = contract['response_schemas'].get(runtime['success_status'])
            if documented is None:
                errors.append(f"::error::{method} {path} returns a JSON success body for HTTP {runtime['success_status']} at runtime but documents no success schema in {contract_path}.")
        missing_request_headers = sorted(runtime['request_headers'] - contract['request_headers'])
        for header_name in missing_request_headers:
            errors.append(f"::error::{method} {path} requires request header `{header_name}` at runtime but is missing it from {contract_path}.")
        unexpected_request_headers = sorted(
            (contract['request_headers'] & set(TRACKED_REQUEST_HEADERS)) - runtime['request_headers']
        )
        for header_name in unexpected_request_headers:
            errors.append(
                f"::error::{method} {path} documents request header `{header_name}` but runtime does not require it in {contract_path}."
            )
        for header_name, runtime_header in sorted(runtime['request_header_details'].items()):
            contract_header = contract['request_header_details'].get(header_name)
            if contract_header is None:
                continue
            if runtime_header.get('required') and not contract_header.get('required'):
                errors.append(f"::error::{method} {path} requires request header `{header_name}` at runtime but it is not marked required in {contract_path}.")
            runtime_type = runtime_header.get('schema_type')
            documented_type = contract_header.get('schema_type')
            if runtime_type and not documented_type:
                errors.append(f"::error::{method} {path} uses request header `{header_name}` as type `{runtime_type}` at runtime but does not document a header schema type in {contract_path}.")
            elif runtime_type and documented_type and runtime_type != documented_type:
                errors.append(f"::error::{method} {path} uses request header `{header_name}` as type `{runtime_type}` at runtime but documents `{documented_type}` in {contract_path}.")
        if runtime['success_status']:
            missing_response_headers = sorted(
                runtime['response_headers'] - contract['response_headers'].get(runtime['success_status'], set())
            )
            for header_name in missing_response_headers:
                errors.append(f"::error::{method} {path} returns response header `{header_name}` for HTTP {runtime['success_status']} at runtime but is missing it from {contract_path}.")
            documented_response_header_details = contract['response_header_details'].get(runtime['success_status'], {})
            for header_name in sorted(runtime['response_headers'] - set(missing_response_headers)):
                expected_type = TRACKED_RESPONSE_HEADERS.get(header_name, {}).get('schema_type')
                if not expected_type:
                    continue
                documented_type = documented_response_header_details.get(header_name, {}).get('schema_type')
                if expected_type != documented_type:
                    actual_type = documented_type or '<none>'
                    errors.append(f"::error::{method} {path} returns response header `{header_name}` for HTTP {runtime['success_status']} as type `{expected_type}` at runtime but documents `{actual_type}` in {contract_path}.")
            runtime_cookie_actions = runtime['response_cookie_actions']
            documented_cookie_actions = contract['response_cookie_actions'].get(runtime['success_status'], set())
            if runtime_cookie_actions or documented_cookie_actions:
                if runtime_cookie_actions != documented_cookie_actions:
                    documented = ', '.join(sorted(documented_cookie_actions)) or '<none>'
                    expected = ', '.join(sorted(runtime_cookie_actions)) or '<none>'
                    errors.append(f"::error::{method} {path} returns Set-Cookie actions [{expected}] for HTTP {runtime['success_status']} at runtime but documents [{documented}] in {contract_path}.")
        if runtime['error_codes']:
            documented_error_codes = set()
            for status_code in runtime['error_statuses']:
                documented_error_codes.update(contract['error_example_codes'].get(status_code, set()))
            unexpected_error_codes = documented_error_codes - runtime['error_codes']
            if unexpected_error_codes:
                documented_error_codes = documented_error_codes - unexpected_error_codes
            if documented_error_codes != runtime['error_codes']:
                documented = ', '.join(sorted(documented_error_codes)) or '<none>'
                expected = ', '.join(sorted(runtime['error_codes']))
                errors.append(f"::error::{method} {path} can emit route-scoped ApiError codes [{expected}] at runtime but documents [{documented}] across tracked error examples in {contract_path}.")
        if runtime['tracked_error_example_codes']:
            documented_error_codes = set().union(*contract['error_example_codes'].values()) if contract['error_example_codes'] else set()
            missing_error_codes = runtime['tracked_error_example_codes'] - documented_error_codes
            if missing_error_codes:
                missing = ', '.join(sorted(missing_error_codes))
                errors.append(f"::error::{method} {path} is missing tracked route-level error examples for ApiError codes [{missing}] in {contract_path}.")
        if runtime['tracked_error_example_codes_by_status']:
            for status_code, expected_codes in sorted(runtime['tracked_error_example_codes_by_status'].items()):
                documented_error_codes = contract['error_example_codes'].get(status_code, set())
                missing_error_codes = expected_codes - documented_error_codes
                if missing_error_codes:
                    missing = ', '.join(sorted(missing_error_codes))
                    errors.append(f"::error::{method} {path} is missing tracked HTTP {status_code} route-level error examples for ApiError codes [{missing}] in {contract_path}.")
        missing_path_parameters = sorted(set(runtime['path_param_names']) - contract['path_parameters'])
        for parameter_name in missing_path_parameters:
            errors.append(f"::error::{method} {path} uses path parameter `{parameter_name}` at runtime but is missing an `in: path` parameter in {contract_path}.")
        for parameter_name, runtime_path in sorted(runtime['path_parameter_details'].items()):
            contract_path_parameter = contract['path_parameter_details'].get(parameter_name)
            if contract_path_parameter is None:
                continue
            if not contract_path_parameter.get('required'):
                errors.append(f"::error::{method} {path} uses path parameter `{parameter_name}` at runtime but it is not marked required in {contract_path}.")
            runtime_type = runtime_path.get('schema_type')
            documented_type = contract_path_parameter.get('schema_type')
            if runtime_type and not documented_type:
                errors.append(f"::error::{method} {path} uses path parameter `{parameter_name}` as type `{runtime_type}` at runtime but does not document a path schema type in {contract_path}.")
            elif runtime_type and documented_type and runtime_type != documented_type:
                errors.append(f"::error::{method} {path} uses path parameter `{parameter_name}` as type `{runtime_type}` at runtime but documents `{documented_type}` in {contract_path}.")
            runtime_format = runtime_path.get('format')
            documented_format = contract_path_parameter.get('format')
            if runtime_format and runtime_format != documented_format:
                actual_format = documented_format or '<none>'
                errors.append(f"::error::{method} {path} uses path parameter `{parameter_name}` with format `{runtime_format}` at runtime but documents `{actual_format}` in {contract_path}.")
        missing_query_parameters = sorted(set(runtime['query_parameters']) - contract['query_parameters'])
        for parameter_name in missing_query_parameters:
            errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` at runtime but is missing an `in: query` parameter in {contract_path}.")
        for parameter_name, runtime_query in sorted(runtime['query_parameters'].items()):
            contract_query = contract['query_parameter_details'].get(parameter_name)
            if contract_query is None:
                continue
            if runtime_query.get('required') and not contract_query.get('required'):
                errors.append(f"::error::{method} {path} requires query parameter `{parameter_name}` at runtime but it is not marked required in {contract_path}.")
            if not runtime_query.get('required') and contract_query.get('required'):
                errors.append(f"::error::{method} {path} treats query parameter `{parameter_name}` as optional at runtime but documents it as required in {contract_path}.")
            runtime_type = runtime_query.get('schema_type')
            documented_type = contract_query.get('schema_type')
            if runtime_type and not documented_type:
                errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` as type `{runtime_type}` at runtime but does not document a query schema type in {contract_path}.")
            elif runtime_type and documented_type and runtime_type != documented_type:
                errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` as type `{runtime_type}` at runtime but documents `{documented_type}` in {contract_path}.")
            runtime_enum = set(runtime_query.get('enum', ()))
            if runtime_enum and runtime_enum != contract_query.get('enum', set()):
                documented = ', '.join(sorted(contract_query.get('enum', set()))) or '<none>'
                expected = ', '.join(sorted(runtime_enum))
                errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` with enum [{expected}] at runtime but documents [{documented}] in {contract_path}.")
            for bound_name in ('minimum', 'maximum'):
                runtime_bound = runtime_query.get(bound_name)
                documented_bound = contract_query.get(bound_name)
                if runtime_bound is not None and runtime_bound != documented_bound:
                    errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` with {bound_name} `{runtime_bound}` at runtime but documents `{documented_bound}` in {contract_path}.")
            runtime_pattern = runtime_query.get('pattern')
            documented_pattern = contract_query.get('pattern')
            if runtime_pattern and runtime_pattern != documented_pattern:
                actual_pattern = documented_pattern or '<none>'
                errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` with pattern `{runtime_pattern}` at runtime but documents `{actual_pattern}` in {contract_path}.")
            runtime_semantics = set(runtime_query.get('semantics', ()))
            documented_semantics = set(contract_query.get('semantics', set()))
            if runtime_semantics and runtime_semantics != documented_semantics:
                documented = ', '.join(sorted(documented_semantics)) or '<none>'
                expected = ', '.join(sorted(runtime_semantics))
                errors.append(f"::error::{method} {path} uses query parameter `{parameter_name}` with semantics [{expected}] at runtime but documents [{documented}] in {contract_path}.")
        missing_error_responses = sorted(runtime['error_statuses'] - contract['error_responses'])
        for status_code in missing_error_responses:
            errors.append(f"::error::{method} {path} can return HTTP {status_code} at runtime but is missing that error response in {contract_path}.")
        tracked_error_schema_statuses = sorted(runtime['error_statuses'] & TRACKED_ERROR_SCHEMA_STATUSES)
        for status_code in tracked_error_schema_statuses:
            documented = contract['response_schemas'].get(status_code)
            if documented != API_ERROR_SCHEMA_NAME:
                actual = documented or '<none>'
                errors.append(
                    f"::error::{method} {path} can return HTTP {status_code} with ApiError at runtime but documents schema `{actual}` instead of `{API_ERROR_SCHEMA_NAME}` in {contract_path}."
                )
        if runtime['success_status'] and runtime['success_status'] not in contract['success_responses']:
            errors.append(f"::error::{method} {path} returns HTTP {runtime['success_status']} at runtime but is missing that success response in {contract_path}.")

    if errors:
        print("\n".join(errors))
        return 1

    return 0

