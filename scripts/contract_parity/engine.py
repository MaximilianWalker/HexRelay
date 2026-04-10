from __future__ import annotations

import pathlib
import re


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
    TRACKED_ERROR_STATUS_TOKENS = {
        '400': ('bad_request(',),
        '403': ('forbidden(',),
        '404': ('StatusCode::NOT_FOUND',),
        '409': ('conflict(',),
        '429': ('too_many_requests(',),
    }
    TRACKED_ERROR_STATUS_PATTERN = '|'.join(sorted(TRACKED_ERROR_STATUS_TOKENS))
    TRACKED_ERROR_EXAMPLE_STATUS_PATTERN = '|'.join(sorted(set(TRACKED_ERROR_STATUS_TOKENS) | {'401'}))
    TRACKED_ERROR_SCHEMA_STATUSES = {'400', '401', '403', '404', '409', '429', '500'}
    API_ERROR_SCHEMA_NAME = 'ApiError'
    AUTH_SESSION_SECURITY_SCHEMES = {'CookieAuth', 'BearerAuth'}
    INTERNAL_TOKEN_REQUIRED_HEADERS = {'x-hexrelay-internal-token'}
    AUTH_PARAM_MARKERS = (
        'AuthSession',
        'AuthorizedServerMembership',
        'AuthorizedServerChannelMembership',
    )
    AUTHORIZER_ERROR_STATUSES = {
        'AuthorizedServerMembership': {'403'},
        'AuthorizedServerChannelMembership': {'403', '404'},
    }
    UNAUTHORIZED_HELPER_NAME_PATTERN = re.compile(r'^(?:map_.*|.*_failure)$')
    TRACKED_REQUEST_HEADERS = {
        'x-hexrelay-internal-token': {
            'runtime_marker': '.get("x-hexrelay-internal-token")',
            'contract_parameter': 'x-hexrelay-internal-token',
        },
    }
    TRACKED_RESPONSE_HEADERS = {
        'Set-Cookie': {
            'runtime_markers': ('append_cookie(',),
        },
    }
    TRACKED_RESPONSE_COOKIE_ACTIONS = {
        'issue:hexrelay_session': (
            r'build_session_cookie_value\(\s*session_cookie_name\(\)',
        ),
        'issue:hexrelay_csrf': (
            r'build_session_cookie_value\(\s*csrf_cookie_name\(\)',
        ),
        'clear:hexrelay_session': (
            r'build_expired_cookie\(\s*session_cookie_name\(\)',
        ),
        'clear:hexrelay_csrf': (
            r'build_expired_cookie\(\s*csrf_cookie_name\(\)',
        ),
    }
    ROUTE_SCOPED_ERROR_CODE_ROUTES = {
        ('POST', '/v1/servers/{server_id}/channels/{channel_id}/messages'),
        ('PATCH', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
        ('DELETE', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
        ('POST', '/v1/dm/threads/{thread_id}/read'),
    }
    ROUTE_SCOPED_ERROR_EXAMPLE_ROUTES = {
        ('POST', '/v1/identity/keys/register'),
        ('POST', '/v1/auth/challenge'),
        ('POST', '/v1/auth/sessions/revoke'),
        ('GET', '/v1/friends/requests'),
        ('POST', '/v1/friends/requests'),
        ('POST', '/v1/friends/requests/{request_id}/accept'),
        ('POST', '/v1/friends/requests/{request_id}/decline'),
        ('POST', '/v1/friends/requests/{request_id}/cancel'),
        ('GET', '/v1/friends/requests/{request_id}/bootstrap'),
        ('POST', '/v1/users/block'),
        ('POST', '/v1/users/mute'),
        ('GET', '/v1/internal/presence/watchers/{identity_id}'),
        ('POST', '/v1/dm/privacy-policy'),
        ('POST', '/v1/dm/pairing-envelope'),
        ('POST', '/v1/dm/pairing-envelope/import'),
        ('POST', '/v1/dm/connectivity/preflight'),
        ('POST', '/v1/dm/connectivity/lan-discovery/announce'),
        ('POST', '/v1/dm/connectivity/wan-wizard'),
        ('POST', '/v1/dm/connectivity/endpoint-cards'),
        ('POST', '/v1/dm/connectivity/endpoint-cards/revoke'),
        ('POST', '/v1/dm/connectivity/parallel-dial'),
        ('POST', '/v1/dm/profile-devices/heartbeat'),
        ('POST', '/v1/dm/fanout/dispatch'),
        ('POST', '/v1/dm/fanout/catch-up'),
        ('POST', '/v1/invites'),
        ('POST', '/v1/contact-invites'),
        ('POST', '/v1/auth/verify'),
        ('POST', '/v1/invites/redeem'),
        ('POST', '/v1/contact-invites/redeem'),
        ('GET', '/v1/discovery/users'),
        ('GET', '/v1/servers/{server_id}'),
        ('GET', '/v1/servers/{server_id}/channels'),
        ('POST', '/v1/servers/{server_id}/channels/{channel_id}/messages'),
        ('PATCH', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
        ('DELETE', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
        ('GET', '/v1/servers/{server_id}/channels/{channel_id}/messages'),
        ('GET', '/v1/dm/threads'),
        ('GET', '/v1/dm/threads/{thread_id}/messages'),
        ('POST', '/v1/dm/threads/{thread_id}/read'),
    }
    ROUTE_SCOPED_ERROR_EXAMPLE_EXPECTATIONS = {
        ('POST', '/v1/identity/keys/register'): {
            'algorithm_invalid',
            'identity_invalid',
            'public_key_invalid',
            'identity_registration_disabled',
            'identity_exists',
        },
        ('POST', '/v1/auth/challenge'): {'identity_invalid'},
        ('POST', '/v1/auth/sessions/revoke'): {'session_invalid'},
        ('GET', '/v1/friends/requests'): {'identity_invalid'},
        ('POST', '/v1/friends/requests'): {
            'identity_invalid',
            'blocked_user',
            'friend_request_exists',
        },
        ('POST', '/v1/friends/requests/{request_id}/accept'): {'identity_invalid', 'transition_invalid'},
        ('POST', '/v1/friends/requests/{request_id}/decline'): {'identity_invalid', 'transition_invalid'},
        ('POST', '/v1/friends/requests/{request_id}/cancel'): {'identity_invalid', 'transition_invalid'},
        ('GET', '/v1/friends/requests/{request_id}/bootstrap'): {
            'identity_invalid',
            'bootstrap_not_available',
            'blocked_user',
        },
        ('POST', '/v1/users/block'): {'identity_invalid', 'already_blocked'},
        ('POST', '/v1/users/mute'): {'identity_invalid', 'already_muted'},
        ('POST', '/v1/dm/privacy-policy'): {'dm_policy_invalid'},
        ('POST', '/v1/dm/pairing-envelope'): {'pairing_invalid'},
        ('POST', '/v1/dm/pairing-envelope/import'): {
            'pairing_invalid',
            'pairing_expired',
            'pairing_replayed',
            'identity_invalid',
        },
        ('POST', '/v1/dm/connectivity/preflight'): {'preflight_invalid'},
        ('POST', '/v1/dm/connectivity/lan-discovery/announce'): {'lan_discovery_invalid'},
        ('POST', '/v1/dm/connectivity/wan-wizard'): {'wan_wizard_invalid'},
        ('POST', '/v1/dm/connectivity/endpoint-cards'): {'endpoint_cards_invalid'},
        ('POST', '/v1/dm/connectivity/endpoint-cards/revoke'): {'endpoint_cards_invalid'},
        ('POST', '/v1/dm/connectivity/parallel-dial'): {'parallel_dial_invalid'},
        ('POST', '/v1/dm/profile-devices/heartbeat'): {'profile_device_invalid'},
        ('POST', '/v1/invites'): {'invite_invalid'},
        ('POST', '/v1/contact-invites'): {'invite_invalid'},
        ('POST', '/v1/auth/verify'): {'identity_invalid', 'nonce_invalid', 'signature_invalid'},
        ('POST', '/v1/invites/redeem'): {
            'invite_invalid',
            'fingerprint_mismatch',
            'invite_expired',
            'invite_exhausted',
        },
        ('POST', '/v1/contact-invites/redeem'): {
            'invite_invalid',
            'invite_expired',
            'invite_exhausted',
            'blocked_user',
            'friend_request_exists',
        },
        ('GET', '/v1/discovery/users'): {'scope_invalid'},
        ('GET', '/v1/servers/{server_id}'): {'server_access_denied'},
        ('GET', '/v1/servers/{server_id}/channels'): {'server_access_denied'},
        ('POST', '/v1/servers/{server_id}/channels/{channel_id}/messages'): {
            'message_content_invalid',
            'reply_target_invalid',
            'mention_invalid',
            'server_access_denied',
            'channel_not_found',
        },
        ('PATCH', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
            'message_content_invalid',
            'mention_invalid',
            'server_access_denied',
            'message_edit_forbidden',
            'channel_not_found',
            'message_not_found',
            'message_deleted',
        },
        ('DELETE', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
            'server_access_denied',
            'message_delete_forbidden',
            'channel_not_found',
            'message_not_found',
        },
        ('GET', '/v1/servers/{server_id}/channels/{channel_id}/messages'): {
            'server_access_denied',
            'channel_not_found',
        },
        ('GET', '/v1/dm/threads'): {'cursor_invalid'},
        ('GET', '/v1/dm/threads/{thread_id}/messages'): {'cursor_invalid', 'thread_not_found'},
        ('POST', '/v1/dm/threads/{thread_id}/read'): {'last_read_seq_invalid', 'thread_not_found'},
        ('POST', '/v1/dm/fanout/dispatch'): {'fanout_invalid'},
        ('POST', '/v1/dm/fanout/catch-up'): {'fanout_invalid', 'cursor_out_of_range'},
    }
    ROUTE_SCOPED_ERROR_EXAMPLE_STATUS_EXPECTATIONS = {
        ('POST', '/v1/identity/keys/register'): {
            '400': {'algorithm_invalid', 'identity_invalid', 'public_key_invalid'},
            '403': {'identity_registration_disabled'},
            '409': {'identity_exists'},
        },
        ('POST', '/v1/auth/challenge'): {
            '400': {'identity_invalid'},
        },
        ('POST', '/v1/auth/verify'): {
            '400': {'identity_invalid', 'nonce_invalid', 'signature_invalid'},
            '401': {'nonce_invalid'},
        },
        ('POST', '/v1/auth/sessions/revoke'): {
            '400': {'session_invalid'},
            '401': {'session_invalid'},
        },
        ('GET', '/v1/friends/requests'): {
            '400': {'identity_invalid'},
            '401': {'identity_invalid'},
        },
        ('POST', '/v1/friends/requests'): {
            '400': {'identity_invalid'},
            '401': {'identity_invalid'},
            '403': {'blocked_user'},
            '409': {'friend_request_exists'},
        },
        ('POST', '/v1/users/block'): {
            '400': {'identity_invalid'},
            '409': {'already_blocked'},
        },
        ('POST', '/v1/users/mute'): {
            '400': {'identity_invalid'},
            '409': {'already_muted'},
        },
        ('GET', '/v1/internal/presence/watchers/{identity_id}'): {
            '401': {'internal_token_invalid'},
        },
        ('POST', '/v1/servers/{server_id}/channels/{channel_id}/messages'): {
            '400': {
                'message_content_invalid',
                'reply_target_invalid',
                'mention_invalid',
            },
            '403': {'server_access_denied'},
            '404': {'channel_not_found'},
        },
        ('PATCH', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
            '400': {'message_content_invalid', 'mention_invalid'},
            '403': {'server_access_denied', 'message_edit_forbidden'},
            '404': {'channel_not_found', 'message_not_found'},
            '409': {'message_deleted'},
        },
        ('DELETE', '/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
            '403': {'server_access_denied', 'message_delete_forbidden'},
            '404': {'channel_not_found', 'message_not_found'},
        },
    }
    QUERY_RUNTIME_FIELD_RULES = {
        'FriendRequestListQuery': {
            'direction': {'enum': ('inbound', 'outbound')},
        },
        'ServerChannelMessageListQuery': {
            'limit': {'minimum': 1, 'maximum': 100},
        },
        'DmThreadListQuery': {
            'limit': {'minimum': 1, 'maximum': 100},
        },
        'DmThreadMessageListQuery': {
            'limit': {'minimum': 1, 'maximum': 100},
        },
        'ServerListQuery': {
            'search': {
                'schema_type': 'string',
                'required': False,
                'semantics': ('blank-means-omitted', 'case-insensitive'),
            },
            'favorites_only': {'schema_type': 'boolean', 'required': False},
            'unread_only': {'schema_type': 'boolean', 'required': False},
            'muted_only': {'schema_type': 'boolean', 'required': False},
        },
        'ContactListQuery': {
            'search': {
                'schema_type': 'string',
                'required': False,
                'semantics': ('blank-means-omitted', 'case-insensitive'),
            },
            'online_only': {'schema_type': 'boolean', 'required': False},
            'unread_only': {'schema_type': 'boolean', 'required': False},
            'favorites_only': {'schema_type': 'boolean', 'required': False},
        },
        'DiscoveryUserListQuery': {
            'scope': {
                'schema_type': 'string',
                'required': False,
                'enum': ('global', 'shared_server'),
                'semantics': ('default:global', 'trim-before-enum'),
            },
            'query': {
                'schema_type': 'string',
                'required': False,
                'semantics': ('blank-means-omitted', 'case-insensitive'),
            },
            'limit': {
                'schema_type': 'integer',
                'required': False,
                'minimum': 1,
                'maximum': 50,
                'semantics': ('default:20', 'clamp:1:50'),
            },
        },
    }
    REQUEST_SCHEMA_ALIASES = {}
    RESPONSE_SCHEMA_ALIASES = {
        'ServerChannelMessageRecord': 'ServerChannelMessage',
    }


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
                    'request_body_schema': extract_request_body_schema(params),
                    'response_body_schema': extract_response_body_schema(return_type),
                    'request_headers': extract_runtime_request_headers(params, body),
                    'response_headers': extract_runtime_response_headers(body),
                    'response_cookie_actions': extract_runtime_response_cookie_actions(body),
                    'has_path_params': 'Path<' in params,
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


    def extract_request_body_schema(params: str):
        match = re.search(r'(?:^|[^A-Za-z0-9_:])Json\s*\(?.*?:\s*Json<\s*([^>]+)\s*>', params, re.S)
        if not match:
            return None
        raw_type = match.group(1).strip()
        normalized = raw_type.split('::')[-1]
        return REQUEST_SCHEMA_ALIASES.get(normalized, normalized)


    def extract_response_body_schema(return_type: str):
        json_match = re.search(r'Json<\s*([^>]+)\s*>', return_type)
        if not json_match:
            return None
        raw_type = json_match.group(1).strip()
        normalized = raw_type.split('::')[-1]
        return RESPONSE_SCHEMA_ALIASES.get(normalized, normalized)


    def extract_runtime_request_headers(params: str, body: str):
        headers = set()
        has_header_map = 'HeaderMap' in params
        for header_name, rule in TRACKED_REQUEST_HEADERS.items():
            if has_header_map and rule['runtime_marker'] in body:
                headers.add(header_name)
        return headers


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


    def extract_query_struct_fields(models_path: pathlib.Path):
        text = models_path.read_text()
        structs = {}
        struct_pattern = re.compile(r'pub struct\s+(\w+)\s*\{(.*?)\n\}', re.S)
        field_pattern = re.compile(r'pub\s+(\w+):\s*([^,\n]+)')

        for match in struct_pattern.finditer(text):
            name = match.group(1)
            body = match.group(2)
            if not name.endswith('Query'):
                continue
            field_details = {}
            for field_name, raw_type in field_pattern.findall(body):
                schema_type, required = map_query_schema_type(raw_type.strip())
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
            path_param_names = sorted(set(re.findall(r'\{([A-Za-z0-9_]+)\}', path)))
            for method, handler in re.findall(r'\b(get|post|put|patch|delete)\s*\(\s*(\w+)\s*\)', block):
                handler_id = route_handler_lookup.get(handler)
                handler_semantics = function_semantics.get(handler_id, {})
                query_type = handler_semantics.get('query_type')
                semantics[(method.upper(), path)] = {
                                'handler': handler,
                                'has_auth': bool(handler_semantics.get('has_auth')),
                                'has_internal_auth': bool(handler_semantics.get('has_internal_auth')),
                                'has_500': bool(handler_semantics.get('has_auth'))
                                or infer_has_500(handler_id, function_semantics, local_lookup),
                    'has_csrf': bool(handler_semantics.get('has_csrf')),
                    'has_json_body': bool(handler_semantics.get('has_json_body')),
                    'request_body_schema': handler_semantics.get('request_body_schema'),
                    'response_body_schema': handler_semantics.get('response_body_schema'),
                    'success_body_kind': infer_success_body_kind(
                        handler_id, function_semantics, local_lookup
                    ),
                    'request_headers': handler_semantics.get('request_headers', set()),
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


    def extract_contract_semantics(contract_path: pathlib.Path):
        lines = contract_path.read_text().splitlines()
        response_components = {}
        semantics = {}
        in_paths = False
        current_path = None
        current_method = None
        current_parameter_in = None
        current_parameter_name = None
        current_query_schema_parameter = None
        in_request_body = False
        in_request_body_json = False
        in_request_body_schema = False
        current_response_status = None
        in_response_json = False
        in_response_schema = False
        in_parameters_block = False
        in_response_headers = False
        current_response_header_status = None
        current_response_header_name = None
        in_response_cookie_actions = False
        current_error_status = None
        in_error_examples = False
        in_error_example_value = False

        current_component_response = None
        in_component_response_json = False
        in_component_response_schema = False
        in_component_responses = False

        for line in lines:
            if not in_component_responses:
                if re.match(r'^  responses:\s*$', line):
                    in_component_responses = True
                continue

            if re.match(r'^  schemas:\s*$', line):
                break

            response_component_match = re.match(r'^    ([A-Za-z0-9_]+):\s*$', line)
            if response_component_match:
                current_component_response = response_component_match.group(1)
                response_components.setdefault(current_component_response, None)
                in_component_response_json = False
                in_component_response_schema = False
                continue

            if current_component_response and re.match(r'^      content:\s*$', line):
                continue
            if current_component_response and re.match(r'^        application/json:\s*$', line):
                in_component_response_json = True
                in_component_response_schema = False
                continue
            if current_component_response and in_component_response_json and re.match(r'^          schema:\s*$', line):
                in_component_response_schema = True
                continue

            component_schema_match = re.match(r"^            \$ref: '#/components/schemas/([A-Za-z0-9_]+)'\s*$", line)
            if current_component_response and in_component_response_schema and component_schema_match:
                response_components[current_component_response] = component_schema_match.group(1)
                continue

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
                current_parameter_name = None
                current_query_schema_parameter = None
                in_request_body = False
                in_request_body_json = False
                in_request_body_schema = False
                current_response_status = None
                in_response_json = False
                in_response_schema = False
                in_parameters_block = False
                in_response_headers = False
                current_response_header_status = None
                current_response_header_name = None
                in_response_cookie_actions = False
                current_error_status = None
                in_error_examples = False
                in_error_example_value = False
                continue

            method_match = re.match(r'^    (get|post|put|patch|delete):\s*$', line)
            if method_match and current_path:
                current_method = method_match.group(1).upper()
                current_parameter_in = None
                current_parameter_name = None
                current_query_schema_parameter = None
                in_request_body = False
                in_request_body_json = False
                in_request_body_schema = False
                current_response_status = None
                in_response_json = False
                in_response_schema = False
                in_parameters_block = False
                in_response_headers = False
                current_response_header_status = None
                current_response_header_name = None
                in_response_cookie_actions = False
                current_error_status = None
                in_error_examples = False
                in_error_example_value = False
                semantics[(current_method, current_path)] = {
                    'security_schemes': set(),
                    'has_401': False,
                    'has_500': False,
                    'has_csrf': False,
                    'has_request_body': False,
                    'request_body_schema': None,
                    'response_schemas': {},
                    'request_headers': set(),
                    'response_headers': {},
                    'response_cookie_actions': {},
                    'error_example_codes': {},
                    'path_parameters': set(),
                    'query_parameters': set(),
                    'query_parameter_details': {},
                    'error_responses': set(),
                    'success_responses': set(),
                }
                continue

            if not current_path or not current_method:
                continue

            if in_request_body_schema and not re.match(r'^ {14,}', line):
                in_request_body_schema = False
            if in_request_body_json and not re.match(r'^ {12,}', line):
                in_request_body_json = False
                in_request_body_schema = False
            if in_request_body and not re.match(r'^ {8,}', line):
                in_request_body = False
                in_request_body_json = False
                in_request_body_schema = False
            if in_response_schema and not re.match(r'^ {14,}', line):
                in_response_schema = False
            if in_response_json and not re.match(r'^ {10,}', line):
                in_response_json = False
                in_response_schema = False
            if current_response_status and re.match(r"^        '(?:4\d\d|5\d\d)':\s*$", line):
                current_response_status = None
                in_response_json = False
                in_response_schema = False
                in_response_headers = False
                current_response_header_status = None
                current_response_header_name = None
                in_response_cookie_actions = False
            elif current_response_status and not re.match(r'^ {8,}', line):
                current_response_status = None
                in_response_json = False
                in_response_schema = False
                in_response_headers = False
                current_response_header_status = None
                current_response_header_name = None
                in_response_cookie_actions = False
            if current_error_status and re.match(r"^        '(?:2\d\d|4\d\d|5\d\d)':\s*$", line) and not re.match(rf"^        '{current_error_status}':\s*$", line):
                current_error_status = None
                in_error_examples = False
                in_error_example_value = False
            if in_response_headers and not re.match(r'^ {10,}', line):
                in_response_headers = False
                current_response_header_name = None
                in_response_cookie_actions = False
            if current_response_header_name and not re.match(r'^ {14,}', line):
                current_response_header_name = None
                in_response_cookie_actions = False
            if in_response_cookie_actions and not re.match(r'^ {16,}', line):
                in_response_cookie_actions = False
            if in_parameters_block and not re.match(r'^ {8,}', line):
                in_parameters_block = False
            if in_error_example_value and not re.match(r'^ {18,}', line):
                in_error_example_value = False
            if in_error_examples and not re.match(r'^ {16,}', line):
                in_error_examples = False
                in_error_example_value = False

            if re.match(r'^      security:\s*$', line):
                continue
            if re.match(r'^      parameters:\s*$', line):
                in_parameters_block = True
                continue

            security_scheme_match = re.match(r'^        - ([A-Za-z0-9_]+): \[\]\s*$', line)
            if security_scheme_match:
                semantics[(current_method, current_path)]['security_schemes'].add(
                    security_scheme_match.group(1)
                )
                continue

            if re.match(r'^      requestBody:\s*$', line):
                semantics[(current_method, current_path)]['has_request_body'] = True
                in_request_body = True
                in_request_body_json = False
                in_request_body_schema = False
                continue
            if in_request_body and re.match(r'^          application/json:\s*$', line):
                in_request_body_json = True
                in_request_body_schema = False
                continue
            if in_request_body_json and re.match(r'^            schema:\s*$', line):
                in_request_body_schema = True
                continue
            request_schema_match = re.match(r"^              \$ref: '#/components/schemas/([A-Za-z0-9_]+)'\s*$", line)
            if in_request_body_schema and request_schema_match:
                semantics[(current_method, current_path)]['request_body_schema'] = request_schema_match.group(1)
                continue

            if "#/components/parameters/CsrfTokenHeader" in line:
                semantics[(current_method, current_path)]['has_csrf'] = True
                continue
            if re.match(r"^        '401':\s*$", line):
                semantics[(current_method, current_path)]['has_401'] = True
            if re.match(r"^        '500':\s*$", line):
                semantics[(current_method, current_path)]['has_500'] = True

            request_header_ref_match = re.match(r"^        - \$ref: '#/components/parameters/([A-Za-z0-9_]+)'\s*$", line)
            if in_parameters_block and request_header_ref_match:
                parameter_ref = request_header_ref_match.group(1)
                if parameter_ref == 'CsrfTokenHeader':
                    semantics[(current_method, current_path)]['request_headers'].add('x-csrf-token')
                continue

            if re.match(r'^      [A-Za-z_][A-Za-z0-9_]*:\s*$', line):
                current_parameter_in = None
                current_parameter_name = None
                current_query_schema_parameter = None

            parameter_match = re.match(r'^        - in: (path|query|header)\s*$', line)
            if parameter_match:
                current_parameter_in = parameter_match.group(1)
                current_parameter_name = None
                current_query_schema_parameter = None
                continue

            other_parameter_match = re.match(r'^        - in: [A-Za-z_][A-Za-z0-9_]*\s*$', line)
            if other_parameter_match:
                current_parameter_in = None
                current_parameter_name = None
                current_query_schema_parameter = None
                continue

            parameter_name_match = re.match(r'^          name: ([A-Za-z0-9_-]+)\s*$', line)
            if parameter_name_match and current_parameter_in in {'path', 'query'}:
                current_parameter_name = parameter_name_match.group(1)
                if current_parameter_in == 'path':
                    semantics[(current_method, current_path)]['path_parameters'].add(current_parameter_name)
                else:
                    semantics[(current_method, current_path)]['query_parameters'].add(current_parameter_name)
                    semantics[(current_method, current_path)]['query_parameter_details'].setdefault(
                        current_parameter_name,
                        {
                            'required': False,
                            'schema_type': None,
                            'enum': set(),
                            'minimum': None,
                            'maximum': None,
                            'semantics': set(),
                        },
                    )
                continue
            if parameter_name_match and current_parameter_in == 'header':
                semantics[(current_method, current_path)]['request_headers'].add(parameter_name_match.group(1))
                continue

            if current_parameter_in == 'query' and current_parameter_name and re.match(r'^          required: true\s*$', line):
                semantics[(current_method, current_path)]['query_parameter_details'][current_parameter_name]['required'] = True
                continue
            if current_parameter_in == 'query' and current_parameter_name and re.match(r'^          schema:\s*$', line):
                current_query_schema_parameter = current_parameter_name
                continue
            if current_query_schema_parameter and not re.match(r'^            ', line):
                current_query_schema_parameter = None
            if current_parameter_in == 'query' and current_parameter_name and re.match(r'^          x-hexrelay-query-semantics:\s*$', line):
                semantics[(current_method, current_path)]['query_parameter_details'][current_parameter_name]['_in_semantics'] = True
                continue
            if current_parameter_in == 'query' and current_parameter_name:
                in_semantics = semantics[(current_method, current_path)]['query_parameter_details'][current_parameter_name].get('_in_semantics', False)
                semantic_match = re.match(r'^            - ([A-Za-z0-9:_-]+)\s*$', line)
                if in_semantics and semantic_match:
                    semantics[(current_method, current_path)]['query_parameter_details'][current_parameter_name]['semantics'].add(semantic_match.group(1))
                    continue
                if in_semantics and not re.match(r'^            ', line):
                    semantics[(current_method, current_path)]['query_parameter_details'][current_parameter_name]['_in_semantics'] = False
            if current_query_schema_parameter:
                type_match = re.match(r'^            type: ([A-Za-z0-9_]+)\s*$', line)
                if type_match:
                    semantics[(current_method, current_path)]['query_parameter_details'][current_query_schema_parameter]['schema_type'] = type_match.group(1)
                    continue
                enum_match = re.match(r'^            enum: \[(.*)\]\s*$', line)
                if enum_match:
                    raw_values = [value.strip() for value in enum_match.group(1).split(',') if value.strip()]
                    semantics[(current_method, current_path)]['query_parameter_details'][current_query_schema_parameter]['enum'] = set(raw_values)
                    continue
                minimum_match = re.match(r'^            minimum: (\d+)\s*$', line)
                if minimum_match:
                    semantics[(current_method, current_path)]['query_parameter_details'][current_query_schema_parameter]['minimum'] = int(minimum_match.group(1))
                    continue
                maximum_match = re.match(r'^            maximum: (\d+)\s*$', line)
                if maximum_match:
                    semantics[(current_method, current_path)]['query_parameter_details'][current_query_schema_parameter]['maximum'] = int(maximum_match.group(1))
                    continue

            success_match = re.match(r"^        '(2\d\d)':\s*$", line)
            if success_match:
                current_response_status = success_match.group(1)
                in_response_json = False
                in_response_schema = False
                current_response_header_status = current_response_status
                in_response_headers = False
                current_error_status = None
                in_error_examples = False
                in_error_example_value = False
                semantics[(current_method, current_path)]['success_responses'].add(current_response_status)
                continue
            error_response_status_match = re.match(r"^        '((?:4\d\d|5\d\d))':\s*$", line)
            if error_response_status_match:
                current_response_status = error_response_status_match.group(1)
                in_response_json = False
                in_response_schema = False
                current_response_header_status = current_response_status
                in_response_headers = False
            if current_response_header_status and re.match(r'^          headers:\s*$', line):
                in_response_headers = True
                continue
            response_header_match = re.match(r'^            ([A-Za-z0-9-]+):\s*$', line)
            if in_response_headers and response_header_match:
                current_response_header_name = response_header_match.group(1)
                in_response_cookie_actions = False
                semantics[(current_method, current_path)]['response_headers'].setdefault(current_response_header_status, set()).add(response_header_match.group(1))
                continue
            if current_response_header_name == 'Set-Cookie' and re.match(r'^              x-hexrelay-cookie-actions:\s*$', line):
                in_response_cookie_actions = True
                semantics[(current_method, current_path)]['response_cookie_actions'].setdefault(current_response_header_status, set())
                continue
            cookie_action_match = re.match(r'^                - ([A-Za-z0-9:_-]+)\s*$', line)
            if in_response_cookie_actions and cookie_action_match:
                semantics[(current_method, current_path)]['response_cookie_actions'].setdefault(current_response_header_status, set()).add(cookie_action_match.group(1))
                continue
            if current_response_status and re.match(r'^ {12}application/json:\s*$', line):
                in_response_json = True
                in_response_schema = False
                continue
            if current_response_status and in_response_json and re.match(r'^ {14}schema:\s*$', line):
                in_response_schema = True
                continue
            response_schema_match = re.match(r"^ {16}\$ref: '#/components/schemas/([A-Za-z0-9_]+)'\s*$", line)
            if current_response_status and in_response_schema and response_schema_match:
                semantics[(current_method, current_path)]['response_schemas'][current_response_status] = response_schema_match.group(1)
                continue
            response_component_ref_match = re.match(r"^ {10}\$ref: '#/components/responses/([A-Za-z0-9_]+)'\s*$", line)
            if current_response_status and response_component_ref_match:
                component_name = response_component_ref_match.group(1)
                component_schema = response_components.get(component_name)
                if component_schema is not None:
                    semantics[(current_method, current_path)]['response_schemas'][current_response_status] = component_schema
                continue

            error_match = re.match(rf"^        '(({TRACKED_ERROR_EXAMPLE_STATUS_PATTERN}))':\s*$", line)
            if error_match:
                current_error_status = error_match.group(1)
                in_error_examples = False
                in_error_example_value = False
                semantics[(current_method, current_path)]['error_responses'].add(current_error_status)
                semantics[(current_method, current_path)]['error_example_codes'].setdefault(current_error_status, set())
                continue
            if current_error_status and re.match(r'^ {14}examples:\s*$', line):
                in_error_examples = True
                in_error_example_value = False
                continue
            if current_error_status and not in_error_examples and current_error_status == '400' and re.match(r'^ {14}\$ref: ', line):
                in_error_examples = False
                in_error_example_value = False
                continue
            if in_error_examples and re.match(r'^ {16}[A-Za-z0-9_]+:\s*$', line):
                in_error_example_value = False
                continue
            if in_error_examples and re.match(r'^ {18}value:\s*$', line):
                in_error_example_value = True
                continue
            error_code_match = re.match(r'^ {20}code: ([A-Za-z0-9_]+)\s*$', line)
            if in_error_example_value and error_code_match:
                semantics[(current_method, current_path)]['error_example_codes'][current_error_status].add(error_code_match.group(1))

        return semantics


    contract_path = pathlib.Path(contract_path_str)
    router_text = pathlib.Path('services/api-rs/src/app/router.rs').read_text()
    handler_paths = sorted(pathlib.Path('services/api-rs/src/transport/http/handlers').glob('*.rs'))
    query_struct_fields = extract_query_struct_fields(pathlib.Path('services/api-rs/src/models.rs'))

    function_semantics, route_handler_lookup, local_lookup = extract_function_blocks(handler_paths)
    runtime_semantics = extract_runtime_semantics(
        router_text, function_semantics, route_handler_lookup, local_lookup, query_struct_fields
    )
    contract_semantics = extract_contract_semantics(contract_path)

    errors = []

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
        if runtime['has_json_body'] and not contract['has_request_body']:
            errors.append(f"::error::{method} {path} accepts a Json request body at runtime but is missing requestBody in {contract_path}.")
        if runtime['request_body_schema'] and contract['request_body_schema'] != runtime['request_body_schema']:
            documented = contract['request_body_schema'] or '<none>'
            errors.append(f"::error::{method} {path} accepts request body schema `{runtime['request_body_schema']}` at runtime but documents `{documented}` in {contract_path}.")
        if runtime['response_body_schema'] and runtime['success_status']:
            documented = contract['response_schemas'].get(runtime['success_status'])
            if documented != runtime['response_body_schema']:
                actual = documented or '<none>'
                errors.append(f"::error::{method} {path} returns response schema `{runtime['response_body_schema']}` for HTTP {runtime['success_status']} at runtime but documents `{actual}` in {contract_path}.")
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
        if runtime['success_status']:
            missing_response_headers = sorted(
                runtime['response_headers'] - contract['response_headers'].get(runtime['success_status'], set())
            )
            for header_name in missing_response_headers:
                errors.append(f"::error::{method} {path} returns response header `{header_name}` for HTTP {runtime['success_status']} at runtime but is missing it from {contract_path}.")
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


def _extract_rust_function_block(text: str, function_name: str) -> str | None:
    match = re.search(
        rf'(?:pub(?:\(crate\))?\s+)?fn\s+{re.escape(function_name)}\b[^{{]*\{{',
        text,
        re.S,
    )
    if not match:
        return None

    start = match.end() - 1
    depth = 0
    for index in range(start, len(text)):
        char = text[index]
        if char == '{':
            depth += 1
        elif char == '}':
            depth -= 1
            if depth == 0:
                return text[start : index + 1]

    return None


def _extract_asyncapi_schema_block(text: str, schema_name: str) -> str | None:
    lines = text.splitlines()
    start = None
    for index, line in enumerate(lines):
        if re.match(rf'^\s{{4}}{re.escape(schema_name)}:\s*$', line):
            start = index
            break

    if start is None:
        return None

    block = []
    for line in lines[start:]:
        if block and re.match(r'^\s{4}[A-Za-z].*:\s*$', line):
            break
        block.append(line)

    return '\n'.join(block)


def _parse_inline_list(block: str, field_name: str) -> list[str]:
    match = re.search(rf'{re.escape(field_name)}:\s*\[([^\]]*)\]', block)
    if not match:
        return []
    return [item.strip() for item in match.group(1).split(',') if item.strip()]


def _extract_top_level_rust_fields(block: str) -> set[str]:
    fields = set()
    depth = 0
    current = []

    for char in block:
        if depth == 0:
            current.append(char)

        if char in '({[':
            depth += 1
        elif char in ')}]':
            depth = max(0, depth - 1)

        if char == '\n':
            line = ''.join(current).strip()
            match = re.match(r'^([a-z_][a-z0-9_]*)\s*:', line)
            if match:
                fields.add(match.group(1))
            current = []

    trailing = ''.join(current).strip()
    match = re.match(r'^([a-z_][a-z0-9_]*)\s*:', trailing)
    if match:
        fields.add(match.group(1))

    return fields


def _parse_realtime_builder(function_block: str) -> dict[str, object] | None:
    envelope_match = re.search(r'RealtimeOutboundEnvelope\s*\{(.*?)\n\s*\};', function_block, re.S)
    if not envelope_match:
        return None

    envelope_block = envelope_match.group(1)
    data_match = re.search(r'data:\s*serde_json::json!\(\s*\{(.*?)\}\s*\)', envelope_block, re.S)
    return {
        'envelope_fields': _extract_top_level_rust_fields(envelope_block),
        'event_type': next(iter(re.findall(r'event_type:\s*"([^"]+)"\.to_string\(\)', envelope_block)), None),
        'event_version': next(iter(re.findall(r'event_version:\s*(\d+)', envelope_block)), None),
        'producer': next(iter(re.findall(r'producer:\s*"([^"]+)"\.to_string\(\)', envelope_block)), None),
        'data_keys': set(re.findall(r'"([^"]+)"\s*:', data_match.group(1) if data_match else '')),
    }


def _parse_rust_struct_fields(runtime_text: str, struct_name: str) -> set[str]:
    struct_match = re.search(rf'struct\s+{re.escape(struct_name)}\s*\{{(.*?)\n\}}', runtime_text, re.S)
    if not struct_match:
        return set()
    return _extract_top_level_rust_fields(struct_match.group(1))


def _parse_realtime_contract_semantics(contract_text: str, event_schema_name: str) -> dict[str, object] | None:
    envelope_block = _extract_asyncapi_schema_block(contract_text, 'EventEnvelopeV1')
    event_block = _extract_asyncapi_schema_block(contract_text, event_schema_name)
    if envelope_block is None or event_block is None:
        return None

    data_ref_match = re.search(r"data:\s*\{\s*\$ref:\s*'#\/components\/schemas\/([^']+)'\s*\}", event_block)
    if not data_ref_match:
        return None

    data_block = _extract_asyncapi_schema_block(contract_text, data_ref_match.group(1))
    if data_block is None:
        return None

    event_type_match = re.search(r'event_type:\s*\{\s*const:\s*([^\s}]+)\s*\}', event_block)
    return {
        'envelope_required': set(_parse_inline_list(envelope_block, 'required')),
        'event_type': event_type_match.group(1) if event_type_match else None,
        'data_required': set(_parse_inline_list(data_block, 'required')),
    }


def _extract_asyncapi_operation_block(text: str, operation_name: str) -> str | None:
    lines = text.splitlines()
    start = None
    for index, line in enumerate(lines):
        if re.match(rf'^\s{{2}}{re.escape(operation_name)}:\s*$', line):
            start = index
            break

    if start is None:
        return None

    block = []
    for line in lines[start:]:
        if block and re.match(r'^\s{2}[A-Za-z].*:\s*$', line):
            break
        block.append(line)

    return '\n'.join(block)


def _parse_asyncapi_flag_list(block: str, field_name: str) -> set[str]:
    lines = block.splitlines()
    values = set()
    in_field = False

    for line in lines:
        if re.match(rf'^\s{{6}}{re.escape(field_name)}:\s*$', line):
            in_field = True
            continue

        if in_field:
            item_match = re.match(r'^\s{8}-\s+([A-Za-z0-9:_-]+)\s*$', line)
            if item_match:
                values.add(item_match.group(1))
                continue

            if not re.match(r'^\s{8}', line):
                break

    return values


def _parse_asyncapi_bool_field(block: str, field_name: str) -> bool | None:
    match = re.search(rf'^\s{{6}}{re.escape(field_name)}:\s*(true|false)\s*$', block, re.M)
    if not match:
        return None
    return match.group(1) == 'true'


def _parse_realtime_signal_contract_semantics(contract_text: str, operation_name: str) -> dict[str, object] | None:
    operation_block = _extract_asyncapi_operation_block(contract_text, operation_name)
    if operation_block is None:
        return None

    semantics_block_match = re.search(r'^\s{4}x-hexrelay-signaling-semantics:\s*$', operation_block, re.M)
    if not semantics_block_match:
        return None

    return {
        'requires_session_identity_from_match': _parse_asyncapi_bool_field(
            operation_block,
            'requires_session_identity_from_match',
        ),
        'supported_targeting': _parse_asyncapi_flag_list(operation_block, 'supported_targeting'),
        'rejection_codes': _parse_asyncapi_flag_list(operation_block, 'rejection_codes'),
    }


def _parse_signal_runtime_semantics(function_block: str, event_name: str) -> dict[str, object] | None:
    event_case_match = re.search(
        rf'"{re.escape(event_name)}"\s*=>\s*(.*?)(?=\n\s*"[^"]+"\s*=>|\n\s*_\s*=>)',
        function_block,
        re.S,
    )
    if not event_case_match:
        return None

    event_case = event_case_match.group(1)
    rejection_codes = set(re.findall(r'build_error_event\(\s*"([^"]+)"', event_case))
    return {
        'requires_session_identity_from_match': bool(
            re.search(r'\bfrom_identity_id\s*!=\s*session_identity_id\b', event_case)
        ),
        'supports_self_targeting_only': bool(
            re.search(r'\bto_identity_id\s*!=\s*session_identity_id\b', event_case)
            and 'event_unsupported' in rejection_codes
        ),
        'rejection_codes': rejection_codes,
    }


def _extract_signal_payload_struct(runtime_text: str, event_name: str) -> str | None:
    route_fn = _extract_rust_function_block(runtime_text, 'route_inbound_event')
    if route_fn is None:
        return None

    event_case_match = re.search(
        rf'"{re.escape(event_name)}"\s*=>\s*(.*?)(?=\n\s*"[^"]+"\s*=>|\n\s*_\s*=>)',
        route_fn,
        re.S,
    )
    if not event_case_match:
        return None

    payload_match = re.search(r'serde_json::from_value::<([A-Za-z0-9_]+)>\(parsed\.data\)', event_case_match.group(1))
    if not payload_match:
        return None

    return payload_match.group(1)


def validate_realtime_semantic_contracts(contract_path_str: str, runtime_path_str: str) -> int:
    tracked_events = {
        'error': {
            'runtime_fn': 'build_error_event',
            'contract_schema': 'ErrorEventV1',
        },
        'call.signal.offer': {
            'runtime_fn': 'build_event',
            'contract_schema': 'CallSignalOfferEventV1',
            'function_args': 'call.signal.offer',
        },
        'call.signal.answer': {
            'runtime_fn': 'build_event',
            'contract_schema': 'CallSignalAnswerEventV1',
            'function_args': 'call.signal.answer',
        },
        'call.signal.ice_candidate': {
            'runtime_fn': 'build_event',
            'contract_schema': 'CallSignalIceCandidateEventV1',
            'function_args': 'call.signal.ice_candidate',
        },
        'realtime.connected': {
            'runtime_fn': 'connection_ready_banner',
            'contract_schema': 'RealtimeConnectedEventV1',
        },
        'presence.updated': {
            'runtime_fn': 'build_presence_updated_event',
            'contract_schema': 'PresenceUpdatedEventV1',
        },
        'channel.message.created': {
            'runtime_fn': 'build_channel_message_created_event',
            'contract_schema': 'ChannelMessageCreatedEventV1',
        },
        'channel.message.updated': {
            'runtime_fn': 'build_channel_message_updated_event',
            'contract_schema': 'ChannelMessageUpdatedEventV1',
        },
        'channel.message.deleted': {
            'runtime_fn': 'build_channel_message_deleted_event',
            'contract_schema': 'ChannelMessageDeletedEventV1',
        },
    }

    contract_path = pathlib.Path(contract_path_str)
    runtime_path = pathlib.Path(runtime_path_str)
    contract_text = contract_path.read_text()
    runtime_text = runtime_path.read_text()
    runtime_inventory = set(extract_realtime_runtime_events(runtime_path_str).splitlines())
    contract_inventory = set(extract_asyncapi_contract_events(contract_path_str).splitlines())
    errors = []

    tracked_event_names = set(tracked_events)
    if not (runtime_inventory & tracked_event_names or contract_inventory & tracked_event_names):
        return 0

    envelope_block = _extract_asyncapi_schema_block(contract_text, 'EventEnvelopeV1')
    if envelope_block is None:
        print(f"::error::{contract_path} is missing EventEnvelopeV1 required for realtime semantic validation.")
        return 1

    envelope_required = set(_parse_inline_list(envelope_block, 'required'))
    producer_match = re.search(r'producer:\s*\{\s*type:\s*string,\s*enum:\s*\[([^\]]*)\]', envelope_block)
    allowed_producers = {
        item.strip() for item in (producer_match.group(1).split(',') if producer_match else []) if item.strip()
    }
    version_match = re.search(r'event_version:\s*\{\s*type:\s*integer,\s*enum:\s*\[([^\]]*)\]', envelope_block)
    allowed_versions = {
        item.strip() for item in (version_match.group(1).split(',') if version_match else []) if item.strip()
    }

    for event_name, spec in tracked_events.items():
        if event_name not in runtime_inventory and event_name not in contract_inventory:
            continue

        function_block = _extract_rust_function_block(runtime_text, spec['runtime_fn'])
        if function_block is None:
            errors.append(
                f"::error::Realtime runtime event `{event_name}` is tracked for semantic parity, but `{spec['runtime_fn']}` was not found in {runtime_path}."
            )
            continue

        if 'function_args' in spec:
            function_block = function_block.replace('event_type.to_string()', f'"{spec["function_args"]}".to_string()')
            function_block = function_block.replace('data,', 'data: serde_json::json!({}),')

        if event_name == 'error':
            function_block = re.sub(r'code:\s*code\.to_string\(\)', 'code: "event_invalid".to_string()', function_block)
            function_block = re.sub(r'message:\s*message\.to_string\(\)', 'message: "invalid event envelope payload".to_string()', function_block)

        runtime_semantics = _parse_realtime_builder(function_block)
        if runtime_semantics is None:
            errors.append(
                f"::error::Realtime runtime event `{event_name}` is tracked for semantic parity, but `{spec['runtime_fn']}` does not build a parseable RealtimeOutboundEnvelope in {runtime_path}."
            )
            continue

        contract_semantics = _parse_realtime_contract_semantics(contract_text, spec['contract_schema'])
        if contract_semantics is None:
            errors.append(
                f"::error::Realtime runtime event `{event_name}` is tracked for semantic parity, but {contract_path} is missing the schema wiring for `{spec['contract_schema']}`."
            )
            continue

        runtime_envelope_fields = runtime_semantics['envelope_fields']
        if runtime_envelope_fields != envelope_required:
            documented = ', '.join(sorted(envelope_required)) or '<none>'
            actual = ', '.join(sorted(runtime_envelope_fields)) or '<none>'
            errors.append(
                f"::error::Realtime runtime event `{event_name}` uses envelope fields [{actual}] but documents [{documented}] in {contract_path}."
            )

        if runtime_semantics['event_type'] != contract_semantics['event_type']:
            errors.append(
                f"::error::Realtime runtime event `{event_name}` emits event_type `{runtime_semantics['event_type']}` but documents `{contract_semantics['event_type']}` in {contract_path}."
            )

        if runtime_semantics['event_version'] not in allowed_versions:
            documented = ', '.join(sorted(allowed_versions)) or '<none>'
            errors.append(
                f"::error::Realtime runtime event `{event_name}` emits event_version `{runtime_semantics['event_version']}` but documents [{documented}] in {contract_path}."
            )

        if runtime_semantics['producer'] not in allowed_producers:
            documented = ', '.join(sorted(allowed_producers)) or '<none>'
            errors.append(
                f"::error::Realtime runtime event `{event_name}` emits producer `{runtime_semantics['producer']}` but documents [{documented}] in {contract_path}."
            )

        runtime_data_keys = runtime_semantics['data_keys']
        if event_name == 'error' and not runtime_data_keys:
            runtime_data_keys = _parse_rust_struct_fields(runtime_text, 'RealtimeErrorData')
        elif 'function_args' in spec and not runtime_data_keys:
            payload_struct = _extract_signal_payload_struct(runtime_text, event_name)
            if payload_struct:
                runtime_data_keys = _parse_rust_struct_fields(runtime_text, payload_struct)
        documented_data_keys = contract_semantics['data_required']
        if runtime_data_keys != documented_data_keys:
            documented = ', '.join(sorted(documented_data_keys)) or '<none>'
            actual = ', '.join(sorted(runtime_data_keys)) or '<none>'
            errors.append(
                f"::error::Realtime runtime event `{event_name}` uses data fields [{actual}] but documents [{documented}] in {contract_path}."
            )

    if errors:
        print('\n'.join(errors))
        return 1

    return 0


def validate_realtime_signal_semantics(contract_path_str: str, runtime_path_str: str) -> int:
    tracked_events = {
        'call.signal.offer': {
            'contract_operation': 'sendCallSignalOffer',
        },
        'call.signal.answer': {
            'contract_operation': 'sendCallSignalAnswer',
        },
        'call.signal.ice_candidate': {
            'contract_operation': 'sendCallSignalIceCandidate',
        },
    }

    contract_path = pathlib.Path(contract_path_str)
    runtime_path = pathlib.Path(runtime_path_str)
    contract_text = contract_path.read_text()
    runtime_text = runtime_path.read_text()
    runtime_inventory = set(extract_realtime_runtime_events(runtime_path_str).splitlines())
    contract_inventory = set(extract_asyncapi_contract_events(contract_path_str).splitlines())
    route_fn = _extract_rust_function_block(runtime_text, 'route_inbound_event')
    errors = []

    tracked_event_names = set(tracked_events)
    if not (runtime_inventory & tracked_event_names or contract_inventory & tracked_event_names):
        return 0

    if route_fn is None:
        print(f"::error::{runtime_path} is missing route_inbound_event required for signaling semantic validation.")
        return 1

    for event_name, spec in tracked_events.items():
        if event_name not in runtime_inventory and event_name not in contract_inventory:
            continue

        runtime_semantics = _parse_signal_runtime_semantics(route_fn, event_name)
        if runtime_semantics is None:
            errors.append(
                f"::error::Realtime runtime event `{event_name}` is tracked for signaling semantic parity, but route_inbound_event does not expose a parseable branch for it in {runtime_path}."
            )
            continue

        contract_semantics = _parse_realtime_signal_contract_semantics(contract_text, spec['contract_operation'])
        if contract_semantics is None:
            errors.append(
                f"::error::Realtime runtime event `{event_name}` is tracked for signaling semantic parity, but {contract_path} is missing x-hexrelay-signaling-semantics for `{spec['contract_operation']}`."
            )
            continue

        documented_identity_match = contract_semantics['requires_session_identity_from_match']
        if documented_identity_match is None:
            errors.append(
                f"::error::Realtime runtime event `{event_name}` is tracked for signaling semantic parity, but `{spec['contract_operation']}` is missing `requires_session_identity_from_match` in {contract_path}."
            )
        elif runtime_semantics['requires_session_identity_from_match'] != documented_identity_match:
            expected = 'requires' if runtime_semantics['requires_session_identity_from_match'] else 'does not require'
            documented = 'requires' if documented_identity_match else 'does not require'
            errors.append(
                f"::error::Realtime runtime event `{event_name}` {expected} from_identity_id/session-identity parity at runtime but {documented} it in {contract_path}."
            )

        runtime_targeting = {'self_only'} if runtime_semantics['supports_self_targeting_only'] else {'recipient_delivery'}
        documented_targeting = contract_semantics['supported_targeting']
        if runtime_targeting != documented_targeting:
            documented = ', '.join(sorted(documented_targeting)) or '<none>'
            actual = ', '.join(sorted(runtime_targeting)) or '<none>'
            errors.append(
                f"::error::Realtime runtime event `{event_name}` supports targeting [{actual}] but documents [{documented}] in {contract_path}."
            )

        documented_rejections = contract_semantics['rejection_codes']
        runtime_rejections = runtime_semantics['rejection_codes']
        missing_rejections = runtime_semantics['rejection_codes'] - documented_rejections
        if missing_rejections:
            missing = ', '.join(sorted(missing_rejections))
            errors.append(
                f"::error::Realtime runtime event `{event_name}` can reject with [{missing}] at runtime but {contract_path} omits them from `{spec['contract_operation']}` signaling semantics."
            )

        extra_rejections = documented_rejections - runtime_rejections
        if extra_rejections:
            extra = ', '.join(sorted(extra_rejections))
            errors.append(
                f"::error::Realtime runtime event `{event_name}` documents rejection codes [{extra}] in `{spec['contract_operation']}` signaling semantics, but the runtime branch cannot emit them in {contract_path}."
            )

    if errors:
        print('\n'.join(errors))
        return 1

    return 0

def extract_realtime_runtime_events(path: str) -> str:
    text = pathlib.Path(path).read_text()
    events = set(re.findall(r'"(call\.signal\.[^"]+)"\s*=>', text))
    events.update(re.findall(r'event_type:\s*"([^"]+)"\.to_string\(\)', text))
    return "\n".join(sorted(events))

def extract_asyncapi_contract_events(path: str) -> str:
    lines = pathlib.Path(path).read_text().splitlines()
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

    return "\n".join(sorted(events))

def extract_realtime_runtime_error_codes(*paths: str) -> str:
    codes = set()
    for path in paths:
        text = pathlib.Path(path).read_text()
        codes.update(re.findall(r'build_error_event\(\s*"([^"]+)"', text, re.S))
        codes.update(re.findall(r'ws_rejection\(\s*[^,]+,\s*"([^"]+)"', text, re.S))

    return "\n".join(sorted(codes))

def extract_asyncapi_contract_error_codes(path: str) -> str:
    lines = pathlib.Path(path).read_text().splitlines()
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

    return "\n".join(sorted(set(codes)))
