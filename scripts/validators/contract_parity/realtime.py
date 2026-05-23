from __future__ import annotations

import pathlib
import re

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
        'producer': next(iter(re.findall(r'producer:\s*"([^"]+)"\.to_string\(\)', envelope_block)), None),
        'data_keys': set(re.findall(r'"([^"]+)"\s*:', data_match.group(1) if data_match else '')),
    }


def _parse_rust_struct_fields(runtime_text: str, struct_name: str) -> set[str]:
    struct_match = re.search(rf'struct\s+{re.escape(struct_name)}\s*\{{(.*?)\n\}}', runtime_text, re.S)
    if not struct_match:
        return set()
    return _extract_top_level_rust_fields(struct_match.group(1))


def _parse_realtime_contract_semantics(contract_text: str, event_schema_name: str) -> dict[str, object] | None:
    envelope_block = _extract_asyncapi_schema_block(contract_text, 'EventEnvelope')
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
            'contract_schema': 'ErrorEvent',
        },
        'call.signal.offer': {
            'runtime_fn': 'build_event',
            'contract_schema': 'CallSignalOfferEvent',
            'function_args': 'call.signal.offer',
        },
        'call.signal.answer': {
            'runtime_fn': 'build_event',
            'contract_schema': 'CallSignalAnswerEvent',
            'function_args': 'call.signal.answer',
        },
        'call.signal.ice_candidate': {
            'runtime_fn': 'build_event',
            'contract_schema': 'CallSignalIceCandidateEvent',
            'function_args': 'call.signal.ice_candidate',
        },
        'realtime.connected': {
            'runtime_fn': 'connection_ready_banner',
            'contract_schema': 'RealtimeConnectedEvent',
        },
        'presence.updated': {
            'runtime_fn': 'build_presence_updated_event',
            'contract_schema': 'PresenceUpdatedEvent',
        },
        'channel.message.created': {
            'runtime_fn': 'build_channel_message_created_event',
            'contract_schema': 'ChannelMessageCreatedEvent',
        },
        'channel.message.updated': {
            'runtime_fn': 'build_channel_message_updated_event',
            'contract_schema': 'ChannelMessageUpdatedEvent',
        },
        'channel.message.deleted': {
            'runtime_fn': 'build_channel_message_deleted_event',
            'contract_schema': 'ChannelMessageDeletedEvent',
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

    envelope_block = _extract_asyncapi_schema_block(contract_text, 'EventEnvelope')
    if envelope_block is None:
        print(f"::error::{contract_path} is missing EventEnvelope required for realtime semantic validation.")
        return 1

    envelope_required = set(_parse_inline_list(envelope_block, 'required'))
    producer_match = re.search(r'producer:\s*\{\s*type:\s*string,\s*enum:\s*\[([^\]]*)\]', envelope_block)
    allowed_producers = {
        item.strip() for item in (producer_match.group(1).split(',') if producer_match else []) if item.strip()
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

def extract_realtime_runtime_events(*paths: str) -> str:
    events = set()
    for path in paths:
        runtime_path = pathlib.Path(path)
        if not runtime_path.exists():
            continue

        text = runtime_path.read_text()
        events.update(re.findall(r'"(call\.signal\.[^"]+)"\s*=>', text))
        events.update(re.findall(r'event_type\s*(?:==|!=)\s*"([^"]+)"', text))
        events.update(re.findall(r'event_type:\s*"([^"]+)"\.to_string\(\)', text))
        events.update(re.findall(r'const\s+[A-Z0-9_]*EVENT_TYPE\s*:\s*&str\s*=\s*"([^"]+)"', text))
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
        runtime_path = pathlib.Path(path)
        if not runtime_path.exists():
            continue

        text = runtime_path.read_text()
        codes.update(re.findall(r'build_error_event\(\s*"([^"]+)"', text, re.S))
        codes.update(re.findall(r'code:\s*"([^"]+)"', text, re.S))
        codes.update(re.findall(r'ws_rejection\(\s*[^,]+,\s*"([^"]+)"', text, re.S))

    return "\n".join(sorted(codes))

def extract_asyncapi_contract_error_codes(path: str) -> str:
    lines = pathlib.Path(path).read_text().splitlines()
    codes = []
    in_error_schema = False
    in_code_enum = False

    for line in lines:
        if not in_error_schema:
            if re.match(r'^\s{4}ErrorData:\s*$', line):
                in_error_schema = True
            continue

        if in_error_schema and re.match(r'^\s{4}[A-Za-z].*:\s*$', line) and not re.match(r'^\s{4}ErrorData:\s*$', line):
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
