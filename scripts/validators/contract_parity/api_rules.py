from __future__ import annotations

import re

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
API_ERROR_REQUIRED_FIELDS = {'code', 'message'}
API_ERROR_FIELD_TYPES = {
    'code': 'string',
    'message': 'string',
}
JSON_REQUEST_MEDIA_TYPE = 'application/json'
CSRF_HEADER_NAME = 'x-csrf-token'
CSRF_HEADER_SCHEMA_TYPE = 'string'
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
        'required': True,
        'schema_type': 'string',
    },
}
TRACKED_RESPONSE_HEADERS = {
    'Set-Cookie': {
        'runtime_markers': ('append_cookie(',),
        'schema_type': 'string',
    },
}
TRACKED_RESPONSE_COOKIE_ACTIONS = {
    'issue:hexrelay_session': (
        r'build_session_cookie_value\(\s*session_cookie_name\(\)',
        r'build_cookie\(\s*SESSION_COOKIE_NAME\b',
    ),
    'issue:hexrelay_csrf': (
        r'build_session_cookie_value\(\s*csrf_cookie_name\(\)',
        r'build_cookie\(\s*CSRF_COOKIE_NAME\b',
    ),
    'clear:hexrelay_session': (
        r'build_expired_cookie\(\s*session_cookie_name\(\)',
    ),
    'clear:hexrelay_csrf': (
        r'build_expired_cookie\(\s*csrf_cookie_name\(\)',
    ),
}
TRACKED_REST_SCHEMA_NAMES = {
    'AuthVerifyRequest',
    'AuthVerifyResponse',
    'SessionValidateResponse',
    'InviteCreateRequest',
    'InviteCreateResponse',
    'ServerChannelMessageCreateRequest',
    'ServerChannelMessageEditRequest',
    'FriendRequestCreateRequest',
    'DmPolicy',
    'DmPolicyUpdate',
    'DmFanoutDispatchRequest',
    'DmFanoutDispatchResponse',
    'DmFanoutCatchUpRequest',
    'DmFanoutCatchUpItem',
    'DmFanoutCatchUpResponse',
    'DmThreadMarkReadRequest',
    'DmThreadMarkReadResponse',
}
IDENTITY_ID_PATTERN = r'^[A-Za-z0-9_-]{3,64}$'
TRACKED_REST_SCHEMA_FIELD_CONSTRAINTS = {
    'InviteCreateRequest': {
        'max_uses': {'minimum': 1},
    },
    'ServerChannelMessageCreateRequest': {
        'content': {'min_length': 1, 'max_length': 4000},
    },
    'ServerChannelMessageEditRequest': {
        'content': {'min_length': 1, 'max_length': 4000},
    },
    'DmFanoutDispatchRequest': {
        'message_id': {'min_length': 1, 'max_length': 128},
        'ciphertext': {'min_length': 1, 'max_length': 8192},
        'source_device_id': {'min_length': 1, 'max_length': 64},
        'destination_server_id': {'min_length': 1, 'max_length': 128},
    },
    'DmFanoutCatchUpRequest': {
        'device_id': {'min_length': 1, 'max_length': 64},
        'device_secret': {'min_length': 16, 'max_length': 128},
        'limit': {'minimum': 1, 'maximum': 100},
    },
    'DmThreadMarkReadRequest': {
        'last_read_seq': {'minimum': 0},
    },
    'DmThreadMarkReadResponse': {
        'last_read_seq': {'minimum': 0},
        'unread': {'minimum': 0},
    },
}
TRACKED_REST_SCHEMA_FIELD_ENUMS = {
    'InviteCreateRequest': {
        'mode': ('one_time', 'multi_use'),
    },
    'InviteCreateResponse': {
        'mode': ('one_time', 'multi_use'),
    },
    'DmFanoutDispatchResponse': {
        'status': ('accepted', 'blocked'),
        'reason_code': (
            'fanout_pending_delivery',
            'fanout_forwarded_to_static_peer',
            'fanout_policy_blocked',
            'fanout_same_server_context_required',
            'fanout_blocked_user',
        ),
        'transport_profile': ('encrypted_envelope_server',),
        'delivery_state': ('pending_delivery', 'forwarded', 'rejected'),
        'reachability_state': ('unreachable', 'blocked', 'unknown'),
    },
    'DmFanoutCatchUpResponse': {
        'status': ('ready', 'blocked'),
        'reason_code': (
            'fanout_catch_up_ok',
            'fanout_catch_up_no_missed',
            'fanout_device_unknown',
            'fanout_device_inactive',
        ),
        'transport_profile': ('encrypted_envelope_server',),
    },
    'DmPolicy': {
        'inbound_policy': ('friends_only', 'same_server', 'anyone'),
        'offline_delivery_mode': ('encrypted_envelope_catchup',),
    },
    'DmPolicyUpdate': {
        'inbound_policy': ('friends_only', 'same_server', 'anyone'),
    },
}
TRACKED_REST_SCHEMA_FIELD_FORMATS = {
    'AuthVerifyResponse': {
        'expires_at': 'date-time',
    },
    'SessionValidateResponse': {
        'expires_at': 'date-time',
    },
    'InviteCreateRequest': {
        'expires_at': 'date-time',
    },
    'InviteCreateResponse': {
        'expires_at': 'date-time',
        'created_at': 'date-time',
    },
}
TRACKED_REST_SCHEMA_FIELD_PATTERNS = {
    'AuthVerifyRequest': {
        'identity_id': IDENTITY_ID_PATTERN,
    },
    'FriendRequestCreateRequest': {
        'requester_identity_id': IDENTITY_ID_PATTERN,
        'target_identity_id': IDENTITY_ID_PATTERN,
    },
    'DmFanoutDispatchRequest': {
        'recipient_identity_id': IDENTITY_ID_PATTERN,
    },
    'DmFanoutCatchUpRequest': {
        'device_secret': r'^[A-Za-z0-9_-]{16,128}$',
    },
}
TRACKED_REST_SCHEMA_FIELD_ITEM_PATTERNS = {
    'ServerChannelMessageCreateRequest': {
        'mention_identity_ids': IDENTITY_ID_PATTERN,
    },
    'ServerChannelMessageEditRequest': {
        'mention_identity_ids': IDENTITY_ID_PATTERN,
    },
}
REST_SCHEMA_CONSTRAINT_LABELS = (
    ('min_length', 'minLength'),
    ('max_length', 'maxLength'),
    ('minimum', 'minimum'),
    ('maximum', 'maximum'),
)
ROUTE_SCOPED_ERROR_CODE_ROUTES = {
    ('POST', '/servers/{server_id}/channels/{channel_id}/messages'),
    ('PATCH', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
    ('DELETE', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
    ('POST', '/dm/threads/{thread_id}/read'),
}
ROUTE_SCOPED_ERROR_EXAMPLE_ROUTES = {
    ('POST', '/identity/keys/register'),
    ('POST', '/auth/challenge'),
    ('POST', '/auth/sessions/revoke'),
    ('GET', '/friends/requests'),
    ('POST', '/friends/requests'),
    ('POST', '/friends/requests/{request_id}/accept'),
    ('POST', '/friends/requests/{request_id}/decline'),
    ('POST', '/friends/requests/{request_id}/cancel'),
    ('GET', '/friends/requests/{request_id}/bootstrap'),
    ('POST', '/users/block'),
    ('POST', '/users/mute'),
    ('GET', '/internal/presence/watchers/{identity_id}'),
    ('POST', '/dm/privacy-policy'),
    ('POST', '/dm/profile-devices/heartbeat'),
    ('POST', '/dm/fanout/dispatch'),
    ('POST', '/dm/fanout/catch-up'),
    ('POST', '/internal/dm/envelopes/ack'),
    ('POST', '/invites'),
    ('POST', '/auth/verify'),
    ('POST', '/invites/redeem'),
    ('GET', '/discovery/users'),
    ('GET', '/servers/{server_id}'),
    ('GET', '/servers/{server_id}/channels'),
    ('POST', '/servers/{server_id}/channels/{channel_id}/messages'),
    ('PATCH', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
    ('DELETE', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'),
    ('GET', '/servers/{server_id}/channels/{channel_id}/messages'),
    ('GET', '/dm/threads'),
    ('GET', '/dm/threads/{thread_id}/messages'),
    ('POST', '/dm/threads/{thread_id}/read'),
}
ROUTE_SCOPED_ERROR_EXAMPLE_EXPECTATIONS = {
    ('POST', '/identity/keys/register'): {
        'algorithm_invalid',
        'identity_invalid',
        'public_key_invalid',
        'identity_registration_disabled',
        'identity_exists',
    },
    ('POST', '/auth/challenge'): {'identity_invalid'},
    ('POST', '/auth/sessions/revoke'): {'session_invalid'},
    ('GET', '/friends/requests'): {'identity_invalid'},
    ('POST', '/friends/requests'): {
        'identity_invalid',
        'blocked_user',
        'friend_request_exists',
    },
    ('POST', '/friends/requests/{request_id}/accept'): {'identity_invalid', 'transition_invalid'},
    ('POST', '/friends/requests/{request_id}/decline'): {'identity_invalid', 'transition_invalid'},
    ('POST', '/friends/requests/{request_id}/cancel'): {'identity_invalid', 'transition_invalid'},
    ('GET', '/friends/requests/{request_id}/bootstrap'): {
        'identity_invalid',
        'bootstrap_not_available',
        'blocked_user',
    },
    ('POST', '/users/block'): {'identity_invalid', 'already_blocked'},
    ('POST', '/users/mute'): {'identity_invalid', 'already_muted'},
    ('POST', '/dm/privacy-policy'): {'dm_policy_invalid'},
    ('POST', '/dm/profile-devices/heartbeat'): {'profile_device_invalid'},
    ('POST', '/invites'): {'invite_invalid'},
    ('POST', '/auth/verify'): {'identity_invalid', 'nonce_invalid', 'signature_invalid'},
    ('POST', '/invites/redeem'): {
        'invite_invalid',
        'server_mismatch',
        'invite_expired',
        'invite_exhausted',
    },
    ('GET', '/discovery/users'): {'scope_invalid'},
    ('GET', '/servers/{server_id}'): {'server_access_denied'},
    ('GET', '/servers/{server_id}/channels'): {'server_access_denied'},
    ('POST', '/servers/{server_id}/channels/{channel_id}/messages'): {
        'message_content_invalid',
        'reply_target_invalid',
        'mention_invalid',
        'server_access_denied',
        'channel_not_found',
    },
    ('PATCH', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
        'message_content_invalid',
        'mention_invalid',
        'server_access_denied',
        'message_edit_forbidden',
        'channel_not_found',
        'message_not_found',
        'message_deleted',
    },
    ('DELETE', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
        'server_access_denied',
        'message_delete_forbidden',
        'channel_not_found',
        'message_not_found',
    },
    ('GET', '/servers/{server_id}/channels/{channel_id}/messages'): {
        'server_access_denied',
        'channel_not_found',
    },
    ('GET', '/dm/threads'): {'cursor_invalid'},
    ('GET', '/dm/threads/{thread_id}/messages'): {'cursor_invalid', 'thread_not_found'},
    ('POST', '/dm/threads/{thread_id}/read'): {'last_read_seq_invalid', 'thread_not_found'},
    ('POST', '/dm/fanout/dispatch'): {'fanout_invalid'},
    ('POST', '/dm/fanout/catch-up'): {'fanout_invalid', 'cursor_out_of_range'},
    ('POST', '/internal/dm/envelopes/ack'): {
        'dm_ack_invalid',
        'dm_ack_unknown',
        'internal_token_invalid',
    },
}
ROUTE_SCOPED_ERROR_EXAMPLE_STATUS_EXPECTATIONS = {
    ('POST', '/identity/keys/register'): {
        '400': {'algorithm_invalid', 'identity_invalid', 'public_key_invalid'},
        '403': {'identity_registration_disabled'},
        '409': {'identity_exists'},
    },
    ('POST', '/auth/challenge'): {
        '400': {'identity_invalid'},
    },
    ('POST', '/auth/verify'): {
        '400': {'identity_invalid', 'nonce_invalid', 'signature_invalid'},
        '401': {'nonce_invalid'},
    },
    ('POST', '/auth/sessions/revoke'): {
        '400': {'session_invalid'},
        '401': {'session_invalid'},
    },
    ('GET', '/friends/requests'): {
        '400': {'identity_invalid'},
        '401': {'identity_invalid'},
    },
    ('POST', '/friends/requests'): {
        '400': {'identity_invalid'},
        '401': {'identity_invalid'},
        '403': {'blocked_user'},
        '409': {'friend_request_exists'},
    },
    ('POST', '/users/block'): {
        '400': {'identity_invalid'},
        '409': {'already_blocked'},
    },
    ('POST', '/users/mute'): {
        '400': {'identity_invalid'},
        '409': {'already_muted'},
    },
    ('GET', '/internal/presence/watchers/{identity_id}'): {
        '401': {'internal_token_invalid'},
    },
    ('POST', '/internal/dm/envelopes/ack'): {
        '400': {'dm_ack_invalid', 'dm_ack_unknown'},
        '401': {'internal_token_invalid'},
    },
    ('POST', '/servers/{server_id}/channels/{channel_id}/messages'): {
        '400': {
            'message_content_invalid',
            'reply_target_invalid',
            'mention_invalid',
        },
        '403': {'server_access_denied'},
        '404': {'channel_not_found'},
    },
    ('PATCH', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
        '400': {'message_content_invalid', 'mention_invalid'},
        '403': {'server_access_denied', 'message_edit_forbidden'},
        '404': {'channel_not_found', 'message_not_found'},
        '409': {'message_deleted'},
    },
    ('DELETE', '/servers/{server_id}/channels/{channel_id}/messages/{message_id}'): {
        '403': {'server_access_denied', 'message_delete_forbidden'},
        '404': {'channel_not_found', 'message_not_found'},
    },
}
QUERY_RUNTIME_FIELD_RULES = {
    'FriendRequestListQuery': {
        'identity_id': {'pattern': r'^[A-Za-z0-9_-]{3,64}$'},
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
