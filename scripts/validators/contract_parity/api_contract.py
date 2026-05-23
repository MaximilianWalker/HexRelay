from __future__ import annotations

import pathlib
import re

try:
    from .api_rules import *  # noqa: F403
except ImportError:  # pragma: no cover
    from api_rules import *  # type: ignore  # noqa: F403

def extract_component_request_bodies(lines: list[str]) -> dict[str, dict[str, object]]:
    components: dict[str, dict[str, object]] = {}
    in_request_bodies = False
    current_component = None
    in_request_body_content = False
    in_request_body_json = False
    in_request_body_schema = False

    for line in lines:
        if not in_request_bodies:
            if re.match(r'^  requestBodies:\s*$', line):
                in_request_bodies = True
            continue

        if re.match(r'^  [A-Za-z_][A-Za-z0-9_]*:\s*$', line):
            break

        request_body_component_match = re.match(r'^    ([A-Za-z0-9_]+):\s*$', line)
        if request_body_component_match:
            current_component = request_body_component_match.group(1)
            components[current_component] = {
                'required': False,
                'schema': None,
                'media_types': set(),
            }
            in_request_body_content = False
            in_request_body_json = False
            in_request_body_schema = False
            continue

        if in_request_body_content and not re.match(r'^ {8,}', line):
            in_request_body_content = False
            in_request_body_json = False
            in_request_body_schema = False
        if in_request_body_json and not re.match(r'^ {8,}', line):
            in_request_body_json = False
            in_request_body_schema = False
        if in_request_body_schema and not re.match(r'^ {12,}', line):
            in_request_body_schema = False

        if current_component and re.match(r'^      required:\s+true\s*$', line):
            components[current_component]['required'] = True
            continue
        if current_component and re.match(r'^      content:\s*$', line):
            in_request_body_content = True
            continue
        media_type_match = re.match(r'^        ([^:\s]+/[^:\s]+):\s*$', line)
        if current_component and in_request_body_content and media_type_match:
            media_type = media_type_match.group(1)
            components[current_component]['media_types'].add(media_type)
            in_request_body_json = media_type == JSON_REQUEST_MEDIA_TYPE
            in_request_body_schema = False
            continue
        if current_component and in_request_body_json and re.match(r'^          schema:\s*$', line):
            in_request_body_schema = True
            continue

        request_schema_match = re.match(r"^            \$ref: '#/components/schemas/([A-Za-z0-9_]+)'\s*$", line)
        if current_component and in_request_body_schema and request_schema_match:
            components[current_component]['schema'] = request_schema_match.group(1)
            continue

    return components


def extract_component_parameters(lines: list[str]) -> dict[str, dict[str, object]]:
    components: dict[str, dict[str, object]] = {}
    in_parameters = False
    current_component = None
    in_schema = False

    for line in lines:
        if not in_parameters:
            if re.match(r'^  parameters:\s*$', line):
                in_parameters = True
            continue

        if re.match(r'^  [A-Za-z_][A-Za-z0-9_]*:\s*$', line):
            break

        component_match = re.match(r'^    ([A-Za-z0-9_]+):\s*$', line)
        if component_match:
            current_component = component_match.group(1)
            components[current_component] = {
                'name': None,
                'in': None,
                'required': False,
                'schema_type': None,
            }
            in_schema = False
            continue

        if current_component and in_schema and not re.match(r'^ {8,}', line):
            in_schema = False

        if current_component:
            name_match = re.match(r'^      name:\s+([A-Za-z0-9_-]+)\s*$', line)
            if name_match:
                components[current_component]['name'] = name_match.group(1)
                continue

            location_match = re.match(r'^      in:\s+([A-Za-z0-9_-]+)\s*$', line)
            if location_match:
                components[current_component]['in'] = location_match.group(1)
                continue

            required_match = re.match(r'^      required:\s+(true|false)\s*$', line)
            if required_match:
                components[current_component]['required'] = required_match.group(1) == 'true'
                continue

            if re.match(r'^      schema:\s*$', line):
                in_schema = True
                continue

            type_match = re.match(r'^        type:\s+([A-Za-z0-9_]+)\s*$', line)
            if in_schema and type_match:
                components[current_component]['schema_type'] = type_match.group(1)
                continue

    return components


def extract_component_schema_types(lines: list[str]) -> dict[str, str]:
    schema_types: dict[str, str] = {}
    in_schemas = False
    current_schema = None

    for line in lines:
        if not in_schemas:
            if re.match(r'^  schemas:\s*$', line):
                in_schemas = True
            continue

        if re.match(r'^\S', line):
            break

        schema_match = re.match(r'^    ([A-Za-z0-9_]+):\s*$', line)
        if schema_match:
            current_schema = schema_match.group(1)
            continue

        type_match = re.match(r'^      type:\s+([A-Za-z0-9_]+)\s*$', line)
        if current_schema and type_match:
            schema_types[current_schema] = type_match.group(1)

    return schema_types


def extract_api_error_schema_shape(lines: list[str]) -> dict[str, object]:
    shape = {
        'present': False,
        'schema_type': None,
        'required': set(),
        'field_types': {},
    }
    in_schema = False
    in_required = False
    in_properties = False
    current_property = None

    for line in lines:
        if not in_schema:
            if re.match(rf'^\s{{4}}{API_ERROR_SCHEMA_NAME}:\s*$', line):
                in_schema = True
                shape['present'] = True
            continue

        if re.match(r'^\s{4}[A-Za-z].*:\s*$', line):
            break

        schema_type_match = re.match(r'^\s{6}type:\s+([A-Za-z0-9_]+)\s*$', line)
        if schema_type_match and not in_properties:
            shape['schema_type'] = schema_type_match.group(1)
            continue

        inline_required_match = re.match(r'^\s{6}required:\s*\[(.*)\]\s*$', line)
        if inline_required_match:
            shape['required'] = {
                item.strip()
                for item in inline_required_match.group(1).split(',')
                if item.strip()
            }
            in_required = False
            continue

        if re.match(r'^\s{6}required:\s*$', line):
            in_required = True
            in_properties = False
            current_property = None
            continue

        if in_required:
            required_item_match = re.match(r'^\s{8}-\s+([A-Za-z0-9_]+)\s*$', line)
            if required_item_match:
                shape['required'].add(required_item_match.group(1))
                continue
            if not re.match(r'^\s{8}', line):
                in_required = False

        if re.match(r'^\s{6}properties:\s*$', line):
            in_properties = True
            in_required = False
            current_property = None
            continue

        if in_properties:
            property_match = re.match(r'^\s{8}([A-Za-z0-9_]+):\s*$', line)
            if property_match:
                current_property = property_match.group(1)
                shape['field_types'].setdefault(current_property, None)
                continue

            property_type_match = re.match(r'^\s{10}type:\s+([A-Za-z0-9_]+)\s*$', line)
            if current_property and property_type_match:
                shape['field_types'][current_property] = property_type_match.group(1)
                continue

    return shape


def extract_contract_semantics(contract_path: pathlib.Path):
    lines = contract_path.read_text().splitlines()
    request_body_components = extract_component_request_bodies(lines)
    parameter_components = extract_component_parameters(lines)
    component_schema_types = extract_component_schema_types(lines)
    response_components = {}
    semantics = {}
    in_paths = False
    current_path = None
    current_method = None
    current_parameter_in = None
    current_parameter_name = None
    current_path_schema_parameter = None
    current_query_schema_parameter = None
    current_header_schema_parameter = None
    in_request_body = False
    in_request_body_content = False
    in_request_body_json = False
    in_request_body_schema = False
    current_response_status = None
    in_response_json = False
    in_response_schema = False
    in_parameters_block = False
    in_response_headers = False
    current_response_header_status = None
    current_response_header_name = None
    in_response_header_schema = False
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
            current_path_schema_parameter = None
            current_query_schema_parameter = None
            current_header_schema_parameter = None
            in_request_body = False
            in_request_body_content = False
            in_request_body_json = False
            in_request_body_schema = False
            current_response_status = None
            in_response_json = False
            in_response_schema = False
            in_parameters_block = False
            in_response_headers = False
            current_response_header_status = None
            current_response_header_name = None
            in_response_header_schema = False
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
            current_path_schema_parameter = None
            current_query_schema_parameter = None
            current_header_schema_parameter = None
            in_request_body = False
            in_request_body_content = False
            in_request_body_json = False
            in_request_body_schema = False
            current_response_status = None
            in_response_json = False
            in_response_schema = False
            in_parameters_block = False
            in_response_headers = False
            current_response_header_status = None
            current_response_header_name = None
            in_response_header_schema = False
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
                'request_body_required': False,
                'request_body_schema': None,
                'request_body_media_types': set(),
                'response_schemas': {},
                'request_headers': set(),
                'request_header_details': {},
                'csrf_header': None,
                'response_headers': {},
                'response_header_details': {},
                'response_cookie_actions': {},
                'error_example_codes': {},
                'path_parameters': set(),
                'path_parameter_details': {},
                'query_parameters': set(),
                'query_parameter_details': {},
                'error_responses': set(),
                'success_responses': set(),
            }
            continue

        if not current_path or not current_method:
            continue

        if in_request_body_content and not re.match(r'^ {10,}', line):
            in_request_body_content = False
            in_request_body_json = False
            in_request_body_schema = False
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
            in_response_header_schema = False
            in_response_cookie_actions = False
        elif current_response_status and not re.match(r'^ {8,}', line):
            current_response_status = None
            in_response_json = False
            in_response_schema = False
            in_response_headers = False
            current_response_header_status = None
            current_response_header_name = None
            in_response_header_schema = False
            in_response_cookie_actions = False
        if current_error_status and re.match(r"^        '(?:2\d\d|4\d\d|5\d\d)':\s*$", line) and not re.match(rf"^        '{current_error_status}':\s*$", line):
            current_error_status = None
            in_error_examples = False
            in_error_example_value = False
        if in_response_headers and not re.match(r'^ {10,}', line):
            in_response_headers = False
            current_response_header_name = None
            in_response_header_schema = False
            in_response_cookie_actions = False
        if current_response_header_name and not re.match(r'^ {14,}', line):
            current_response_header_name = None
            in_response_header_schema = False
            in_response_cookie_actions = False
        if in_response_header_schema and not re.match(r'^ {16,}', line):
            in_response_header_schema = False
        if in_response_cookie_actions and not re.match(r'^ {16,}', line):
            in_response_cookie_actions = False
        if in_parameters_block and not re.match(r'^ {8,}', line):
            in_parameters_block = False
        if current_path_schema_parameter and not re.match(r'^            ', line):
            current_path_schema_parameter = None
        if current_query_schema_parameter and not re.match(r'^            ', line):
            current_query_schema_parameter = None
        if current_header_schema_parameter and not re.match(r'^            ', line):
            current_header_schema_parameter = None
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
            in_request_body_content = False
            in_request_body_json = False
            in_request_body_schema = False
            continue
        if in_request_body and re.match(r'^        required:\s+true\s*$', line):
            semantics[(current_method, current_path)]['request_body_required'] = True
            continue
        if in_request_body and re.match(r'^        content:\s*$', line):
            in_request_body_content = True
            continue
        request_media_type_match = re.match(r'^          ([^:\s]+/[^:\s]+):\s*$', line)
        if in_request_body_content and request_media_type_match:
            media_type = request_media_type_match.group(1)
            semantics[(current_method, current_path)]['request_body_media_types'].add(media_type)
            in_request_body_json = media_type == JSON_REQUEST_MEDIA_TYPE
            in_request_body_schema = False
            continue
        if in_request_body_json and re.match(r'^            schema:\s*$', line):
            in_request_body_schema = True
            continue
        request_schema_match = re.match(r"^              \$ref: '#/components/schemas/([A-Za-z0-9_]+)'\s*$", line)
        if in_request_body_schema and request_schema_match:
            semantics[(current_method, current_path)]['request_body_schema'] = request_schema_match.group(1)
            continue
        request_body_component_ref_match = re.match(r"^        \$ref: '#/components/requestBodies/([A-Za-z0-9_]+)'\s*$", line)
        if in_request_body and request_body_component_ref_match:
            component_name = request_body_component_ref_match.group(1)
            component = request_body_components.get(component_name, {})
            semantics[(current_method, current_path)]['request_body_required'] = bool(
                component.get('required')
            )
            component_schema = component.get('schema')
            if component_schema:
                semantics[(current_method, current_path)]['request_body_schema'] = component_schema
            semantics[(current_method, current_path)]['request_body_media_types'] = set(
                component.get('media_types', set())
            )
            continue

        if "#/components/parameters/CsrfTokenHeader" in line:
            csrf_header = parameter_components.get('CsrfTokenHeader', {})
            semantics[(current_method, current_path)]['has_csrf'] = True
            semantics[(current_method, current_path)]['csrf_header'] = csrf_header
            if csrf_header.get('in') == 'header' and csrf_header.get('name'):
                header_name = str(csrf_header.get('name'))
                semantics[(current_method, current_path)]['request_headers'].add(header_name)
                semantics[(current_method, current_path)]['request_header_details'][header_name] = {
                    'required': bool(csrf_header.get('required', False)),
                    'schema_type': csrf_header.get('schema_type'),
                }
            continue
        if re.match(r"^        '401':\s*$", line):
            semantics[(current_method, current_path)]['has_401'] = True
        if re.match(r"^        '500':\s*$", line):
            semantics[(current_method, current_path)]['has_500'] = True

        request_header_ref_match = re.match(r"^        - \$ref: '#/components/parameters/([A-Za-z0-9_]+)'\s*$", line)
        if in_parameters_block and request_header_ref_match:
            parameter_ref = request_header_ref_match.group(1)
            component_parameter = parameter_components.get(parameter_ref, {})
            if component_parameter.get('in') == 'header' and component_parameter.get('name'):
                header_name = str(component_parameter.get('name'))
                semantics[(current_method, current_path)]['request_headers'].add(header_name)
                semantics[(current_method, current_path)]['request_header_details'][header_name] = {
                    'required': bool(component_parameter.get('required', False)),
                    'schema_type': component_parameter.get('schema_type'),
                }
            continue

        if re.match(r'^      [A-Za-z_][A-Za-z0-9_]*:\s*$', line):
            current_parameter_in = None
            current_parameter_name = None
            current_path_schema_parameter = None
            current_query_schema_parameter = None
            current_header_schema_parameter = None

        parameter_match = re.match(r'^        - in: (path|query|header)\s*$', line)
        if parameter_match:
            current_parameter_in = parameter_match.group(1)
            current_parameter_name = None
            current_path_schema_parameter = None
            current_query_schema_parameter = None
            current_header_schema_parameter = None
            continue

        other_parameter_match = re.match(r'^        - in: [A-Za-z_][A-Za-z0-9_]*\s*$', line)
        if other_parameter_match:
            current_parameter_in = None
            current_parameter_name = None
            current_path_schema_parameter = None
            current_query_schema_parameter = None
            current_header_schema_parameter = None
            continue

        parameter_name_match = re.match(r'^          name: ([A-Za-z0-9_-]+)\s*$', line)
        if parameter_name_match and current_parameter_in in {'path', 'query'}:
            current_parameter_name = parameter_name_match.group(1)
            if current_parameter_in == 'path':
                semantics[(current_method, current_path)]['path_parameters'].add(current_parameter_name)
                semantics[(current_method, current_path)]['path_parameter_details'].setdefault(
                    current_parameter_name,
                    {
                        'required': False,
                        'schema_type': None,
                        'format': None,
                    },
                )
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
                        'pattern': None,
                        'semantics': set(),
                    },
                )
            continue
        if parameter_name_match and current_parameter_in == 'header':
            current_parameter_name = parameter_name_match.group(1)
            semantics[(current_method, current_path)]['request_headers'].add(current_parameter_name)
            semantics[(current_method, current_path)]['request_header_details'].setdefault(
                current_parameter_name,
                {
                    'required': False,
                    'schema_type': None,
                },
            )
            continue

        if current_parameter_in == 'path' and current_parameter_name and re.match(r'^          required: true\s*$', line):
            semantics[(current_method, current_path)]['path_parameter_details'][current_parameter_name]['required'] = True
            continue
        if current_parameter_in == 'query' and current_parameter_name and re.match(r'^          required: true\s*$', line):
            semantics[(current_method, current_path)]['query_parameter_details'][current_parameter_name]['required'] = True
            continue
        if current_parameter_in == 'header' and current_parameter_name and re.match(r'^          required: true\s*$', line):
            semantics[(current_method, current_path)]['request_header_details'][current_parameter_name]['required'] = True
            continue
        if current_parameter_in == 'path' and current_parameter_name and re.match(r'^          schema:\s*$', line):
            current_path_schema_parameter = current_parameter_name
            continue
        if current_parameter_in == 'query' and current_parameter_name and re.match(r'^          schema:\s*$', line):
            current_query_schema_parameter = current_parameter_name
            continue
        if current_parameter_in == 'header' and current_parameter_name and re.match(r'^          schema:\s*$', line):
            current_header_schema_parameter = current_parameter_name
            continue
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
        if current_path_schema_parameter:
            type_match = re.match(r'^            type: ([A-Za-z0-9_]+)\s*$', line)
            if type_match:
                semantics[(current_method, current_path)]['path_parameter_details'][current_path_schema_parameter]['schema_type'] = type_match.group(1)
                continue
            format_match = re.match(r'^            format: ([A-Za-z0-9_-]+)\s*$', line)
            if format_match:
                semantics[(current_method, current_path)]['path_parameter_details'][current_path_schema_parameter]['format'] = format_match.group(1)
                continue
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
            pattern_match = re.match(r'^            pattern:\s*(.+?)\s*$', line)
            if pattern_match:
                pattern_value = pattern_match.group(1).strip()
                if (
                    len(pattern_value) >= 2
                    and pattern_value[0] == pattern_value[-1]
                    and pattern_value[0] in {"'", '"'}
                ):
                    pattern_value = pattern_value[1:-1]
                semantics[(current_method, current_path)]['query_parameter_details'][current_query_schema_parameter]['pattern'] = pattern_value
                continue
        if current_header_schema_parameter:
            type_match = re.match(r'^            type: ([A-Za-z0-9_]+)\s*$', line)
            if type_match:
                semantics[(current_method, current_path)]['request_header_details'][current_header_schema_parameter]['schema_type'] = type_match.group(1)
                continue

        success_match = re.match(r"^        '(2\d\d)':\s*$", line)
        if success_match:
            current_response_status = success_match.group(1)
            in_response_json = False
            in_response_schema = False
            current_response_header_status = current_response_status
            in_response_headers = False
            in_response_header_schema = False
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
            in_response_header_schema = False
        if current_response_header_status and re.match(r'^          headers:\s*$', line):
            in_response_headers = True
            continue
        response_header_match = re.match(r'^            ([A-Za-z0-9-]+):\s*$', line)
        if in_response_headers and response_header_match:
            current_response_header_name = response_header_match.group(1)
            in_response_header_schema = False
            in_response_cookie_actions = False
            semantics[(current_method, current_path)]['response_headers'].setdefault(current_response_header_status, set()).add(current_response_header_name)
            semantics[(current_method, current_path)]['response_header_details'].setdefault(
                current_response_header_status,
                {},
            ).setdefault(
                current_response_header_name,
                {
                    'schema_type': None,
                },
            )
            continue
        response_header_inline_ref_match = re.match(r"^              schema:\s*\{\s*\$ref:\s*'#/components/schemas/([A-Za-z0-9_]+)'\s*\}\s*$", line)
        if current_response_header_name and response_header_inline_ref_match:
            schema_name = response_header_inline_ref_match.group(1)
            semantics[(current_method, current_path)]['response_header_details'].setdefault(
                current_response_header_status,
                {},
            ).setdefault(
                current_response_header_name,
                {
                    'schema_type': None,
                },
            )['schema_type'] = component_schema_types.get(schema_name)
            continue
        if current_response_header_name and re.match(r'^              schema:\s*$', line):
            in_response_header_schema = True
            continue
        response_header_type_match = re.match(r'^                type:\s+([A-Za-z0-9_]+)\s*$', line)
        if in_response_header_schema and response_header_type_match:
            semantics[(current_method, current_path)]['response_header_details'].setdefault(
                current_response_header_status,
                {},
            ).setdefault(
                current_response_header_name,
                {
                    'schema_type': None,
                },
            )['schema_type'] = response_header_type_match.group(1)
            continue
        response_header_ref_match = re.match(r"^                \$ref:\s*'#/components/schemas/([A-Za-z0-9_]+)'\s*$", line)
        if in_response_header_schema and response_header_ref_match:
            schema_name = response_header_ref_match.group(1)
            semantics[(current_method, current_path)]['response_header_details'].setdefault(
                current_response_header_status,
                {},
            ).setdefault(
                current_response_header_name,
                {
                    'schema_type': None,
                },
            )['schema_type'] = component_schema_types.get(schema_name)
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
