from __future__ import annotations

import re
from collections.abc import Mapping


INTEGER_RUST_TYPES = {'u8', 'u16', 'u32', 'u64', 'usize', 'i8', 'i16', 'i32', 'i64', 'isize'}


def extract_query_type(params: str):
    match = re.search(
        r'(?:^|[^A-Za-z0-9_:])(?:axum::extract::)?Query\s*\(?.*?:\s*(?:axum::extract::)?Query<\s*([A-Za-z0-9_]+)\s*>',
        params,
        re.S,
    )
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
    elif normalized_type in INTEGER_RUST_TYPES:
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


def extract_request_body_schema(params: str, aliases: Mapping[str, str] | None = None):
    match = re.search(r'(?:^|[^A-Za-z0-9_:])Json\s*\(?.*?:\s*Json<\s*([^>]+)\s*>', params, re.S)
    if not match:
        return None
    raw_type = match.group(1).strip()
    normalized = raw_type.split('::')[-1]
    return (aliases or {}).get(normalized, normalized)


def has_request_body_extractor(params: str):
    return 'Json<' in params or bool(
        re.search(r'(?:^|[^A-Za-z0-9_:])(?:axum::body::)?Bytes\b', params)
    )


def normalize_response_schema(raw_type: str, aliases: Mapping[str, str] | None = None):
    normalized = raw_type.strip().split('::')[-1]
    return (aliases or {}).get(normalized, normalized)


def extract_response_builder_body_schema(
    body: str,
    aliases: Mapping[str, str] | None = None,
):
    local_structs = {
        name: normalize_response_schema(schema_name, aliases)
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


def has_json_response_builder(body: str, aliases: Mapping[str, str] | None = None):
    return extract_response_builder_body_schema(body, aliases) is not None


def extract_response_body_schema(
    return_type: str,
    body: str,
    aliases: Mapping[str, str] | None = None,
):
    json_match = re.search(r'Json<\s*([^>]+)\s*>', return_type)
    if json_match:
        return normalize_response_schema(json_match.group(1), aliases)
    return extract_response_builder_body_schema(body, aliases)
