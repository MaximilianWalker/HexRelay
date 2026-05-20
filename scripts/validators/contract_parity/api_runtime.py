from __future__ import annotations

import pathlib
import re

try:
    from .api_rules import *  # noqa: F403
except ImportError:  # pragma: no cover
    from api_rules import *  # type: ignore  # noqa: F403

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


def extract_auth_semantics(params: str):
    markers = {
        marker for marker in AUTH_PARAM_MARKERS if re.search(rf'\b{marker}\b', params)
    }
    implied_error_statuses = set()
    for marker in markers:
        implied_error_statuses.update(AUTHORIZER_ERROR_STATUSES.get(marker, set()))
    return {
        'has_auth': bool(markers),
        'implied_error_statuses': implied_error_statuses,
    }


def extract_function_blocks(paths):
    functions = {}
    route_handler_lookup = {}
    local_lookup = {}
    pattern = re.compile(r'(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\((.*?)\)\s*->', re.S)
    for path in paths:
        text = pathlib.Path(path).read_text()
        source_path = str(path)
        for match in pattern.finditer(text):
            name = match.group(1)
            params = match.group(2)
            auth_semantics = extract_auth_semantics(params)
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
            function_id = f'{source_path}::{name}'
            local_lookup[(source_path, name)] = function_id
            route_handler_lookup[name] = function_id
            functions[function_id] = {
                'name': name,
                'source_path': source_path,
                'has_auth': auth_semantics['has_auth'],
                'has_internal_auth': '.get("x-hexrelay-internal-token")' in body,
                'has_csrf': 'enforce_csrf_for_cookie_auth(' in body,
                'has_json_body': 'Json<' in params,
                'has_request_body_extractor': has_request_body_extractor(params),
                'request_body_schema': extract_request_body_schema(params),
                'response_body_schema': extract_response_body_schema(return_type, body),
                'request_headers': extract_runtime_request_headers(params, body),
                'response_headers': extract_runtime_response_headers(body),
                'response_cookie_actions': extract_runtime_response_cookie_actions(body),
                'has_path_params': 'Path<' in params,
                'path_param_details': extract_path_param_details(params),
                'query_type': extract_query_type(params),
                'implied_error_statuses': auth_semantics['implied_error_statuses'],
                'error_statuses': set(),
                'return_type': return_type,
                'body': body,
            }
    for function_id in list(functions):
        functions[function_id]['error_statuses'] = infer_error_statuses(
            function_id, functions, local_lookup
        )
    return functions, route_handler_lookup, local_lookup


def extract_query_type(params: str):
    match = re.search(r'(?:^|[^A-Za-z0-9_:])(?:axum::extract::)?Query\s*\(?.*?:\s*(?:axum::extract::)?Query<\s*([A-Za-z0-9_]+)\s*>', params, re.S)
    if match:
        return match.group(1)
    return None


def split_top_level_types(raw_types: str):
    types = []
    current = []
    depth = 0
    for char in raw_types:
        if char in '<(':
            depth += 1
        elif char in '>)':
            depth -= 1
        if char == ',' and depth == 0:
            types.append(''.join(current).strip())
            current = []
            continue
        current.append(char)
    if current:
        types.append(''.join(current).strip())
    return types


def map_path_parameter_type(raw_type: str):
    normalized_type = raw_type.replace('&', '').strip()
    details = {}
    if normalized_type in {'String', 'str'} or normalized_type.endswith(' str'):
        details['schema_type'] = 'string'
    elif normalized_type in {'Uuid', 'uuid::Uuid'}:
        details['schema_type'] = 'string'
        details['format'] = 'uuid'
    elif normalized_type in {'u8', 'u16', 'u32', 'u64', 'usize', 'i8', 'i16', 'i32', 'i64', 'isize'}:
        details['schema_type'] = 'integer'
    elif normalized_type == 'bool':
        details['schema_type'] = 'boolean'
    return details


def extract_path_param_type(params: str):
    match = re.search(
        r'(?:^|[^A-Za-z0-9_:])(?:axum::extract::)?Path\b.*?:\s*(?:axum::extract::)?Path\s*<',
        params,
        re.S,
    )
    if not match:
        return None
    cursor = match.end()
    depth = 1
    raw_type = []
    while cursor < len(params):
        char = params[cursor]
        if char == '<':
            depth += 1
        elif char == '>':
            depth -= 1
            if depth == 0:
                return ''.join(raw_type).strip()
        raw_type.append(char)
        cursor += 1
    return None


def extract_path_param_details(params: str):
    raw_type = extract_path_param_type(params)
    if raw_type is None:
        return []
    if raw_type.startswith('(') and raw_type.endswith(')'):
        raw_types = split_top_level_types(raw_type[1:-1])
    else:
        raw_types = [raw_type]
    return [map_path_parameter_type(path_type) for path_type in raw_types]


def extract_request_body_schema(params: str):
    match = re.search(r'(?:^|[^A-Za-z0-9_:])Json\s*\(?.*?:\s*Json<\s*([^>]+)\s*>', params, re.S)
    if not match:
        return None
    raw_type = match.group(1).strip()
    normalized = raw_type.split('::')[-1]
    return REQUEST_SCHEMA_ALIASES.get(normalized, normalized)


def has_request_body_extractor(params: str):
    return 'Json<' in params or bool(
        re.search(r'(?:^|[^A-Za-z0-9_:])(?:axum::body::)?Bytes\b', params)
    )


def normalize_response_schema(raw_type: str):
    normalized = raw_type.strip().split('::')[-1]
    return RESPONSE_SCHEMA_ALIASES.get(normalized, normalized)


def extract_response_builder_body_schema(body: str):
    local_structs = {
        name: normalize_response_schema(schema_name)
        for name, schema_name in re.findall(
            r'\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Z][A-Za-z0-9_]*)\s*\{',
            body,
            re.S,
        )
    }
    json_response_vars = re.findall(
        r'\bJson\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*\.into_response\s*\(',
        body,
        re.S,
    )
    schemas = {local_structs[var_name] for var_name in json_response_vars if var_name in local_structs}
    if len(schemas) == 1:
        return next(iter(schemas))
    return None


def has_json_response_builder(body: str):
    return extract_response_builder_body_schema(body) is not None


def extract_response_body_schema(return_type: str, body: str):
    json_match = re.search(r'Json<\s*([^>]+)\s*>', return_type)
    if json_match:
        return normalize_response_schema(json_match.group(1))
    return extract_response_builder_body_schema(body)


def extract_runtime_request_headers(params: str, body: str):
    headers = set()
    has_header_map = 'HeaderMap' in params
    for header_name, rule in TRACKED_REQUEST_HEADERS.items():
        if has_header_map and rule['runtime_marker'] in body:
            headers.add(header_name)
    return headers


def build_runtime_request_header_details(headers: set[str]):
    details = {}
    for rule in TRACKED_REQUEST_HEADERS.values():
        header_name = rule['contract_parameter']
        if header_name in headers:
            details[header_name] = {
                'required': bool(rule.get('required', False)),
                'schema_type': rule.get('schema_type'),
            }
    return details


def extract_runtime_response_headers(body: str):
    headers = set()
    for header_name, rule in TRACKED_RESPONSE_HEADERS.items():
        if any(marker in body for marker in rule['runtime_markers']):
            headers.add(header_name)
    return headers


def extract_runtime_response_cookie_actions(body: str):
    actions = set()
    for action, patterns in TRACKED_RESPONSE_COOKIE_ACTIONS.items():
        if any(re.search(pattern, body, re.S) for pattern in patterns):
            actions.add(action)
    return actions


def infer_response_headers(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return set()

    function = functions.get(handler_id)
    if not function:
        return set()

    headers = set(function.get('response_headers', set()))
    body = function.get('body', '')
    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        headers.update(
            infer_response_headers(callee_id, functions, local_lookup, stack | {handler_id})
        )

    return headers


def infer_request_headers(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return set()

    function = functions.get(handler_id)
    if not function:
        return set()

    headers = set(function.get('request_headers', set()))
    body = function.get('body', '')
    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        headers.update(
            infer_request_headers(callee_id, functions, local_lookup, stack | {handler_id})
        )

    return headers


def infer_has_csrf(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return False

    function = functions.get(handler_id)
    if not function:
        return False

    if function.get('has_csrf'):
        return True

    body = function.get('body', '')
    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        if infer_has_csrf(callee_id, functions, local_lookup, stack | {handler_id}):
            return True

    return False


def infer_response_cookie_actions(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return set()

    function = functions.get(handler_id)
    if not function:
        return set()

    actions = set(function.get('response_cookie_actions', set()))
    body = function.get('body', '')
    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        actions.update(
            infer_response_cookie_actions(callee_id, functions, local_lookup, stack | {handler_id})
        )

    return actions


def infer_error_codes(handler_id, functions, local_lookup, stack=None, follow_helpers=True):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return set()

    function = functions.get(handler_id)
    if not function:
        return set()

    body = function.get('body', '')
    codes = set(re.findall(r'\b(?:bad_request|forbidden|conflict)\(\s*"([A-Za-z0-9_]+)"', body))
    codes.update(
        re.findall(
            r'StatusCode::NOT_FOUND\s*,\s*Json\(ApiError\s*\{\s*code:\s*"([A-Za-z0-9_]+)"',
            body,
            re.S,
        )
    )

    if not follow_helpers:
        return codes

    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        if callee_id == handler_id:
            continue
        codes.update(
            infer_error_codes(
                callee_id,
                functions,
                local_lookup,
                stack | {handler_id},
                follow_helpers=follow_helpers,
            )
        )

    return codes


def should_track_route_scoped_error_codes(method: str, path: str):
    return (method, path) in ROUTE_SCOPED_ERROR_CODE_ROUTES


def should_track_route_scoped_error_examples(method: str, path: str):
    return (method, path) in ROUTE_SCOPED_ERROR_EXAMPLE_ROUTES


def expected_route_scoped_error_examples(method: str, path: str):
    return ROUTE_SCOPED_ERROR_EXAMPLE_EXPECTATIONS.get((method, path), set())


def expected_route_scoped_error_examples_by_status(method: str, path: str):
    return ROUTE_SCOPED_ERROR_EXAMPLE_STATUS_EXPECTATIONS.get((method, path), {})


def parse_rust_struct_fields(body: str):
    fields = []
    pending_attrs = []
    field_pattern = re.compile(r'pub\s+(\w+):\s*([^,\n]+)')

    for line in body.splitlines():
        stripped = line.strip()
        if stripped.startswith('#['):
            pending_attrs.append(stripped)
            continue

        field_match = field_pattern.match(stripped)
        if field_match:
            fields.append({
                'name': field_match.group(1),
                'raw_type': field_match.group(2),
                'serde_default': any('serde' in attr and 'default' in attr for attr in pending_attrs),
            })
            pending_attrs = []
            continue

        if stripped:
            pending_attrs = []

    return fields


def extract_query_struct_fields(models_path: pathlib.Path):
    text = models_path.read_text()
    structs = {}
    struct_pattern = re.compile(r'pub struct\s+(\w+)\s*\{(.*?)\n\}', re.S)

    for match in struct_pattern.finditer(text):
        name = match.group(1)
        body = match.group(2)
        if not name.endswith('Query'):
            continue
        field_details = {}
        for field in parse_rust_struct_fields(body):
            field_name = field['name']
            schema_type, required = map_query_schema_type(field['raw_type'].strip())
            if field.get('serde_default'):
                required = False
            field_details[field_name] = {
                'required': required,
                'schema_type': schema_type,
            }
        for field_name, rule in QUERY_RUNTIME_FIELD_RULES.get(name, {}).items():
            if field_name not in field_details:
                continue
            field_details[field_name].update(rule)
        structs[name] = field_details

    return structs


def unwrap_rust_generic(raw_type: str, wrapper: str):
    raw_type = raw_type.strip()
    prefix = f'{wrapper}<'
    if raw_type.startswith(prefix) and raw_type.endswith('>'):
        return raw_type[len(prefix):-1].strip()
    return None


def map_rest_schema_type(raw_type: str):
    required = True
    nullable = False
    inner_type = raw_type.strip()
    option_inner = unwrap_rust_generic(inner_type, 'Option')
    if option_inner is not None:
        required = False
        nullable = True
        inner_type = option_inner

    vec_inner = unwrap_rust_generic(inner_type, 'Vec')
    if vec_inner is not None:
        item_details = map_rest_schema_type(vec_inner)
        details = {
            'required': required,
            'nullable': nullable,
            'schema_type': 'array',
            'item_schema_type': item_details['schema_type'],
        }
        if item_details.get('schema_ref'):
            details['item_schema_ref'] = item_details['schema_ref']
        return details

    schema_type = 'object'
    schema_ref = None
    normalized_type = inner_type.replace('&', '').strip()
    if normalized_type == 'String' or normalized_type.endswith(' str'):
        schema_type = 'string'
    elif normalized_type == 'bool':
        schema_type = 'boolean'
    elif normalized_type in {'u8', 'u16', 'u32', 'u64', 'usize', 'i8', 'i16', 'i32', 'i64', 'isize'}:
        schema_type = 'integer'
    elif normalized_type.split('::')[-1] in TRACKED_REST_SCHEMA_NAMES:
        schema_ref = normalized_type.split('::')[-1]

    details = {
        'required': required,
        'nullable': nullable,
        'schema_type': schema_type,
    }
    if schema_ref:
        details['schema_ref'] = schema_ref
    return details


def extract_tracked_schema_fields(models_path: pathlib.Path):
    text = models_path.read_text()
    structs = {}
    struct_pattern = re.compile(r'pub struct\s+(\w+)\s*\{(.*?)\n\}', re.S)

    for match in struct_pattern.finditer(text):
        name = match.group(1)
        if name not in TRACKED_REST_SCHEMA_NAMES:
            continue
        body = match.group(2)
        fields = {}
        for field in parse_rust_struct_fields(body):
            field_name = field['name']
            fields[field_name] = map_rest_schema_type(field['raw_type'].strip())
            if field.get('serde_default'):
                fields[field_name]['required'] = False
        for field_name, constraints in TRACKED_REST_SCHEMA_FIELD_CONSTRAINTS.get(name, {}).items():
            if field_name in fields:
                fields[field_name].update(constraints)
        for field_name, enum_values in TRACKED_REST_SCHEMA_FIELD_ENUMS.get(name, {}).items():
            if field_name in fields:
                fields[field_name]['enum'] = set(enum_values)
        for field_name, schema_format in TRACKED_REST_SCHEMA_FIELD_FORMATS.get(name, {}).items():
            if field_name in fields:
                fields[field_name]['format'] = schema_format
        for field_name, pattern in TRACKED_REST_SCHEMA_FIELD_PATTERNS.get(name, {}).items():
            if field_name in fields:
                fields[field_name]['pattern'] = pattern
        for field_name, pattern in TRACKED_REST_SCHEMA_FIELD_ITEM_PATTERNS.get(name, {}).items():
            if field_name in fields:
                fields[field_name]['item_pattern'] = pattern
        structs[name] = fields

    return structs


def extract_openapi_tracked_schema_fields(contract_path: pathlib.Path):
    lines = contract_path.read_text().splitlines()
    structs = {}
    current_schema = None
    current_required = set()
    current_properties = {}
    current_property = None
    current_property_items = False
    current_property_enum = False
    in_required = False
    in_properties = False

    def parse_enum_values(raw_values: str):
        return {value.strip().strip('"\'') for value in raw_values.split(',') if value.strip()}

    def build_schema_fields(properties: dict[str, dict[str, object]], required: set[str]):
        return {
            field_name: {
                **properties.get(field_name, {}),
                'required': field_name in required,
                'nullable': bool(properties.get(field_name, {}).get('nullable', False)),
            }
            for field_name in set(properties) | required
        }

    for line in lines:
        schema_match = re.match(r'^    ([A-Za-z0-9_]+):\s*$', line)
        if schema_match:
            if current_schema in TRACKED_REST_SCHEMA_NAMES and current_properties:
                structs[current_schema] = build_schema_fields(current_properties, current_required)
            current_schema = schema_match.group(1)
            current_required = set()
            current_properties = {}
            current_property = None
            current_property_items = False
            current_property_enum = False
            in_required = False
            in_properties = False
            continue

        if current_schema not in TRACKED_REST_SCHEMA_NAMES:
            continue

        if re.match(r'^      required:\s*\[(.*)\]\s*$', line):
            match = re.match(r'^      required:\s*\[(.*)\]\s*$', line)
            current_required = {item.strip() for item in match.group(1).split(',') if item.strip()}
            in_required = False
            continue
        if re.match(r'^      required:\s*$', line):
            in_required = True
            in_properties = False
            continue
        if in_required:
            required_match = re.match(r'^        - ([A-Za-z0-9_]+)\s*$', line)
            if required_match:
                current_required.add(required_match.group(1))
                continue
            if not re.match(r'^        ', line):
                in_required = False

        if re.match(r'^      properties:\s*$', line):
            in_properties = True
            in_required = False
            current_property = None
            current_property_items = False
            continue
        if in_properties:
            property_match = re.match(r'^        ([A-Za-z0-9_]+):\s*$', line)
            if property_match:
                current_property = property_match.group(1)
                current_property_items = False
                current_property_enum = False
                current_properties.setdefault(current_property, {})
                continue
            if current_property:
                if current_property_enum:
                    inline_enum_values = re.match(r'^ {12}\[(.*)\]\s*$', line)
                    enum_item_match = re.match(r'^ {12}-\s*([A-Za-z0-9:_-]+)\s*$', line)
                    if inline_enum_values:
                        current_properties[current_property]['enum'] = parse_enum_values(inline_enum_values.group(1))
                        current_property_enum = False
                        continue
                    if enum_item_match:
                        current_properties[current_property].setdefault('enum', set()).add(enum_item_match.group(1))
                        continue
                    if not re.match(r'^ {12}', line):
                        current_property_enum = False
                type_match = re.match(r'^( {10}| {12})type:\s*([A-Za-z0-9_]+)\s*$', line)
                if type_match:
                    indent = len(type_match.group(1))
                    if current_property_items and indent == 12:
                        current_properties[current_property]['item_schema_type'] = type_match.group(2)
                    elif not current_property_items and indent == 10:
                        current_properties[current_property]['schema_type'] = type_match.group(2)
                    continue
                format_match = re.match(r'^( {10})format:\s*([A-Za-z0-9_.:-]+)\s*$', line)
                if format_match and not current_property_items:
                    current_properties[current_property]['format'] = format_match.group(2)
                    continue
                pattern_match = re.match(r'^( {10})pattern:\s*(.+?)\s*$', line)
                if pattern_match and not current_property_items:
                    pattern_value = pattern_match.group(2).strip()
                    if (
                        len(pattern_value) >= 2
                        and pattern_value[0] == pattern_value[-1]
                        and pattern_value[0] in {"'", '"'}
                    ):
                        pattern_value = pattern_value[1:-1]
                    current_properties[current_property]['pattern'] = pattern_value
                    continue
                item_pattern_match = re.match(r'^( {12})pattern:\s*(.+?)\s*$', line)
                if item_pattern_match and current_property_items:
                    pattern_value = item_pattern_match.group(2).strip()
                    if (
                        len(pattern_value) >= 2
                        and pattern_value[0] == pattern_value[-1]
                        and pattern_value[0] in {"'", '"'}
                    ):
                        pattern_value = pattern_value[1:-1]
                    current_properties[current_property]['item_pattern'] = pattern_value
                    continue
                nullable_match = re.match(r'^( {10})nullable:\s*(true|false)\s*$', line)
                if nullable_match:
                    current_properties[current_property]['nullable'] = nullable_match.group(2) == 'true'
                    current_property_items = False
                    continue
                enum_match = re.match(r'^( {10})enum:\s*(?:\[(.*)\])?\s*$', line)
                if enum_match and not current_property_items:
                    raw_values = enum_match.group(2)
                    if raw_values is None:
                        current_property_enum = True
                        current_properties[current_property].setdefault('enum', set())
                    else:
                        current_properties[current_property]['enum'] = parse_enum_values(raw_values)
                        current_property_enum = False
                    continue
                constraint_match = re.match(r'^( {10})(minLength|maxLength|minimum|maximum):\s*(\d+)\s*$', line)
                if constraint_match and not current_property_items:
                    constraint_key = {
                        'minLength': 'min_length',
                        'maxLength': 'max_length',
                        'minimum': 'minimum',
                        'maximum': 'maximum',
                    }[constraint_match.group(2)]
                    current_properties[current_property][constraint_key] = int(constraint_match.group(3))
                    continue
                if re.match(r'^          items:\s*$', line):
                    current_property_items = True
                    current_property_enum = False
                    continue
                if re.match(r'^ {10,}properties:\s*$', line):
                    current_property = None
                    current_property_items = False
                    current_property_enum = False
                    continue
                schema_ref_match = re.match(r"^( {10}| {12})\$ref: '#/components/schemas/([A-Za-z0-9_]+)'\s*$", line)
                if schema_ref_match:
                    indent = len(schema_ref_match.group(1))
                    schema_name = schema_ref_match.group(2)
                    if current_property_items and indent == 12:
                        current_properties[current_property]['item_schema_type'] = 'object'
                        current_properties[current_property]['item_schema_ref'] = schema_name
                    elif not current_property_items and indent == 10:
                        current_properties[current_property]['schema_type'] = 'object'
                        current_properties[current_property]['schema_ref'] = schema_name
                    continue
                if not re.match(r'^ {10,}', line):
                    current_property = None
                    current_property_items = False
                    current_property_enum = False
            if not re.match(r'^          ', line):
                in_properties = False
                current_property = None
                current_property_items = False
                current_property_enum = False

    if current_schema in TRACKED_REST_SCHEMA_NAMES and current_properties:
        structs[current_schema] = build_schema_fields(current_properties, current_required)

    return structs


def map_query_schema_type(raw_type: str):
    required = True
    inner_type = raw_type
    option_match = re.fullmatch(r'Option<\s*([^>]+)\s*>', raw_type)
    if option_match:
        required = False
        inner_type = option_match.group(1).strip()

    schema_type = 'string'
    if inner_type == 'bool':
        schema_type = 'boolean'
    elif inner_type in {'u8', 'u16', 'u32', 'u64', 'usize', 'i8', 'i16', 'i32', 'i64', 'isize'}:
        schema_type = 'integer'

    return schema_type, required


def infer_error_statuses(handler_id, functions, local_lookup, stack=None, follow_helpers=True):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return set()

    function = functions.get(handler_id)
    if not function:
        return set()

    body = function.get('body', '')
    statuses = set()

    for status, tokens in TRACKED_ERROR_STATUS_TOKENS.items():
        if any(token in body for token in tokens):
            statuses.add(status)

    if not follow_helpers:
        return statuses

    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        if callee_id == handler_id:
            continue
        statuses.update(
            infer_error_statuses(
                callee_id,
                functions,
                local_lookup,
                stack | {handler_id},
                follow_helpers=follow_helpers,
            )
        )

    return statuses


def infer_has_401(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return False

    function = functions.get(handler_id)
    if not function:
        return False

    body = function.get('body', '')
    if 'unauthorized(' in body:
        return True

    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    for callee_id in sorted(helper_ids):
        callee_name = functions[callee_id]['name']
        if not UNAUTHORIZED_HELPER_NAME_PATTERN.match(callee_name):
            continue
        if infer_has_401(callee_id, functions, local_lookup, stack | {handler_id}):
            return True

    return False


def infer_has_400(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return False

    function = functions.get(handler_id)
    if not function:
        return False

    body = function.get('body', '')
    if 'bad_request(' in body:
        return True

    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        if infer_has_400(callee_id, functions, local_lookup, stack | {handler_id}):
            return True

    return False


def infer_has_500(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return False

    function = functions.get(handler_id)
    if not function:
        return False

    body = function.get('body', '')
    if 'internal_error(' in body:
        return True

    helper_ids = resolve_local_helper_ids(body, function, local_lookup)
    helper_ids.update(resolve_local_delegate_ids(body, function, local_lookup))
    for callee_id in sorted(helper_ids):
        if infer_has_500(callee_id, functions, local_lookup, stack | {handler_id}):
            return True

    return False


def resolve_local_helper_ids(body, function, local_lookup):
    helper_names = set(re.findall(r'\b(\w+)\s*\(', body))
    helper_names.update(
        re.findall(r'\b(?:ok_or_else|map_err|or_else)\s*\(\s*(\w+)\s*\)', body)
    )
    source_path = function.get('source_path')
    return {
        local_lookup[(source_path, callee_name)]
        for callee_name in helper_names
        if (source_path, callee_name) in local_lookup
    }


def resolve_local_delegate_ids(body, function, local_lookup):
    source_path = function.get('source_path')
    delegate_names = set(re.findall(r'\b(\w+)\s*\([^;]*\)\.await', body))
    return {
        local_lookup[(source_path, callee_name)]
        for callee_name in delegate_names
        if (source_path, callee_name) in local_lookup
    }


def infer_success_status(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return None

    function = functions.get(handler_id)
    if not function:
        return None

    body = function.get('body', '')
    if 'StatusCode::CREATED' in body:
        return '201'
    if 'StatusCode::NO_CONTENT' in body:
        return '204'
    if 'StatusCode::ACCEPTED' in body:
        return '202'
    if has_json_response_builder(body):
        return '200'

    body_inner = body.strip()
    if body_inner.startswith('{') and body_inner.endswith('}'):
        body_inner = body_inner[1:-1].strip()
    delegate_ids = resolve_local_delegate_ids(body_inner, function, local_lookup)
    if delegate_ids:
        delegated_status = infer_success_status(
            sorted(delegate_ids)[0], functions, local_lookup, stack | {handler_id}
        )
        if delegated_status:
            return delegated_status

    if 'Json<' in function.get('return_type', ''):
        return '200'

    return None


def infer_success_body_kind(handler_id, functions, local_lookup, stack=None):
    if stack is None:
        stack = set()
    if handler_id in stack:
        return None

    function = functions.get(handler_id)
    if not function:
        return None

    return_type = function.get('return_type', '')
    if 'Json<' in return_type:
        return 'json'
    if has_json_response_builder(function.get('body', '')):
        return 'json'
    if 'StatusCode' in return_type:
        return 'none'

    body_inner = function.get('body', '').strip()
    if body_inner.startswith('{') and body_inner.endswith('}'):
        body_inner = body_inner[1:-1].strip()
    delegate_ids = resolve_local_delegate_ids(body_inner, function, local_lookup)
    if delegate_ids:
        delegated_kind = infer_success_body_kind(
            sorted(delegate_ids)[0], functions, local_lookup, stack | {handler_id}
        )
        if delegated_kind:
            return delegated_kind

    return None


def extract_runtime_semantics(router_text: str, function_semantics, route_handler_lookup, local_lookup, query_struct_fields):
    semantics = {}
    for block in route_blocks(router_text):
        path_match = re.search(r'\.route\(\s*"([^"]+)"', block, re.S)
        if not path_match:
            continue
        path = re.sub(r':([A-Za-z0-9_]+)', r'{\1}', path_match.group(1))
        path_param_names = re.findall(r'\{([A-Za-z0-9_]+)\}', path)
        for method, handler in re.findall(r'\b(get|post|put|patch|delete)\s*\(\s*(\w+)\s*\)', block):
            handler_id = route_handler_lookup.get(handler)
            handler_semantics = function_semantics.get(handler_id, {})
            query_type = handler_semantics.get('query_type')
            path_param_details = {}
            if handler_semantics.get('has_path_params'):
                runtime_path_details = handler_semantics.get('path_param_details', [])
                path_param_details = {
                    name: runtime_path_details[index] if index < len(runtime_path_details) else {}
                    for index, name in enumerate(path_param_names)
                }
            inferred_request_headers = infer_request_headers(
                handler_id, function_semantics, local_lookup
            )
            semantics[(method.upper(), path)] = {
                            'handler': handler,
                            'has_auth': bool(handler_semantics.get('has_auth')),
                            'has_internal_auth': bool(handler_semantics.get('has_internal_auth'))
                            or bool(inferred_request_headers & INTERNAL_TOKEN_REQUIRED_HEADERS),
                            'has_500': bool(handler_semantics.get('has_auth'))
                            or infer_has_500(handler_id, function_semantics, local_lookup),
                'has_csrf': infer_has_csrf(handler_id, function_semantics, local_lookup),
                'has_json_body': bool(handler_semantics.get('has_json_body')),
                'has_request_body_extractor': bool(
                    handler_semantics.get('has_request_body_extractor')
                ),
                'request_body_schema': handler_semantics.get('request_body_schema'),
                'response_body_schema': handler_semantics.get('response_body_schema'),
                'success_body_kind': infer_success_body_kind(
                    handler_id, function_semantics, local_lookup
                ),
                'request_headers': inferred_request_headers,
                'request_header_details': build_runtime_request_header_details(
                    inferred_request_headers
                ),
                'response_headers': infer_response_headers(
                    handler_id, function_semantics, local_lookup
                ),
                'response_cookie_actions': infer_response_cookie_actions(
                    handler_id, function_semantics, local_lookup
                ),
                'error_codes': infer_error_codes(
                    handler_id,
                    function_semantics,
                    local_lookup,
                    follow_helpers=method.upper() != 'GET',
                ) if should_track_route_scoped_error_codes(method.upper(), path) else set(),
                'tracked_error_example_codes': expected_route_scoped_error_examples(
                    method.upper(), path
                ) if should_track_route_scoped_error_examples(method.upper(), path) else set(),
                'tracked_error_example_codes_by_status': expected_route_scoped_error_examples_by_status(
                    method.upper(), path
                ) if should_track_route_scoped_error_examples(method.upper(), path) else {},
                'path_param_names': path_param_names if handler_semantics.get('has_path_params') else [],
                'path_parameter_details': path_param_details,
                'query_parameters': query_struct_fields.get(query_type, {}) if query_type else {},
                'error_statuses': infer_error_statuses(
                    handler_id,
                    function_semantics,
                    local_lookup,
                    follow_helpers=method.upper() != 'GET',
                )
                | set(handler_semantics.get('implied_error_statuses', set()))
                | (
                    {'400'}
                    if method.upper() == 'GET'
                    and infer_has_400(handler_id, function_semantics, local_lookup)
                    else set()
                ),
                'has_401': bool(handler_semantics.get('has_auth')) or infer_has_401(
                    handler_id, function_semantics, local_lookup
                ),
                'success_status': infer_success_status(
                    handler_id, function_semantics, local_lookup
                ),
            }
    return semantics
