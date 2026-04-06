#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import subprocess
import sys

try:
    from . import engine
except ImportError:  # pragma: no cover
    import engine  # type: ignore


API_CONTRACT = "docs/contracts/runtime-rest-v1.openapi.yaml"
REALTIME_CONTRACT = "docs/contracts/realtime-events-runtime-v1.asyncapi.yaml"
API_SURFACE_FILES = [
    "services/api-rs/src/models.rs",
    "services/api-rs/src/app/router.rs",
    "services/api-rs/src/domain/**/*.rs",
    "services/api-rs/src/shared/errors.rs",
    "services/api-rs/src/transport/http/middleware/auth.rs",
    "services/api-rs/src/transport/http/middleware/authorization.rs",
    "services/api-rs/src/transport/http/middleware/rate_limit.rs",
    "services/api-rs/src/transport/http/handlers/*.rs",
]
REALTIME_SURFACE_FILES = [
    "services/realtime-rs/src/app/router.rs",
    "services/realtime-rs/src/domain/events/*.rs",
    "services/realtime-rs/src/transport/ws/middleware/rate_limit.rs",
    "services/realtime-rs/src/transport/ws/handlers/*.rs",
]


def run_git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def resolve_base_sha(base_sha: str) -> str:
    if base_sha and base_sha != "0000000000000000000000000000000000000000":
        return base_sha

    candidates = [
        ["merge-base", "HEAD", "origin/main"],
        ["merge-base", "HEAD", "origin/master"],
        ["rev-parse", "HEAD~1"],
    ]
    for candidate in candidates:
        result = subprocess.run(["git", *candidate], check=False, capture_output=True, text=True)
        if result.returncode == 0:
            return result.stdout.strip()
    raise SystemExit("::error::failed to resolve base sha for contract parity validation.")


def git_diff_name_only(base_sha: str, head_sha: str, paths: list[str]) -> str:
    return run_git("diff", "--name-only", base_sha, head_sha, "--", *paths)


def contract_changed(base_sha: str, head_sha: str, path: str) -> bool:
    changed = git_diff_name_only(base_sha, head_sha, [path]).splitlines()
    return path in changed


def compare_inventory(label: str, runtime_inventory: str, contract_inventory: str) -> bool:
    runtime = sorted({line for line in runtime_inventory.splitlines() if line.strip()})
    contract = sorted({line for line in contract_inventory.splitlines() if line.strip()})
    missing_from_contract = [line for line in runtime if line not in contract]
    extra_in_contract = [line for line in contract if line not in runtime]

    if not missing_from_contract and not extra_in_contract:
        return False

    print(f"::error::{label} drift detected between runtime inventory and contract.")
    if missing_from_contract:
        print("Missing from contract:")
        print("\n".join(missing_from_contract))
    if extra_in_contract:
        print("Only in contract:")
        print("\n".join(extra_in_contract))
    return True


def has_required_version_field(path: str, field: str) -> bool:
    contents = pathlib.Path(path).read_text()
    return contents.startswith(f"{field}:") or any(
        line.startswith(f"{field}:") for line in contents.splitlines()
    )


def main(argv: list[str]) -> int:
    base_sha = resolve_base_sha(argv[1] if len(argv) > 1 else "")
    head_sha = argv[2] if len(argv) > 2 else "HEAD"

    api_surface_changes = git_diff_name_only(base_sha, head_sha, API_SURFACE_FILES)
    realtime_surface_changes = git_diff_name_only(base_sha, head_sha, REALTIME_SURFACE_FILES)
    api_contract_changed = contract_changed(base_sha, head_sha, API_CONTRACT)
    realtime_contract_changed = contract_changed(base_sha, head_sha, REALTIME_CONTRACT)

    api_runtime_inventory = engine.extract_api_runtime_inventory("services/api-rs/src/app/router.rs")
    api_contract_inventory = engine.extract_openapi_contract_inventory(API_CONTRACT)
    api_runtime_error_codes = engine.extract_api_runtime_error_codes(
        "services/api-rs/src/domain/**/*.rs",
        "services/api-rs/src/shared/errors.rs",
        "services/api-rs/src/transport/http/middleware/auth.rs",
        "services/api-rs/src/transport/http/middleware/authorization.rs",
        "services/api-rs/src/transport/http/middleware/rate_limit.rs",
        "services/api-rs/src/transport/http/handlers/*.rs",
    )
    api_contract_error_codes = engine.extract_openapi_contract_error_codes(API_CONTRACT)
    realtime_runtime_events = engine.extract_realtime_runtime_events(
        "services/realtime-rs/src/domain/events/service.rs"
    )
    realtime_contract_events = engine.extract_asyncapi_contract_events(REALTIME_CONTRACT)
    realtime_runtime_error_codes = engine.extract_realtime_runtime_error_codes(
        "services/realtime-rs/src/domain/events/service.rs",
        "services/realtime-rs/src/transport/ws/handlers/gateway.rs",
    )
    realtime_contract_error_codes = engine.extract_asyncapi_contract_error_codes(
        REALTIME_CONTRACT
    )

    errors = False

    if api_surface_changes and not api_contract_changed:
        print(f"::error::API contract-surface files changed but {API_CONTRACT} was not updated.")
        print("Changed API surface files:")
        print(api_surface_changes)
        errors = True

    if realtime_surface_changes and not realtime_contract_changed:
        print(f"::error::Realtime websocket/event surface changed but {REALTIME_CONTRACT} was not updated.")
        print("Changed realtime surface files:")
        print(realtime_surface_changes)
        errors = True

    if not has_required_version_field(API_CONTRACT, "openapi"):
        print(f"::error::{API_CONTRACT} is missing required openapi version field.")
        errors = True

    if not has_required_version_field(REALTIME_CONTRACT, "asyncapi"):
        print(f"::error::{REALTIME_CONTRACT} is missing required asyncapi version field.")
        errors = True

    errors = compare_inventory("API route inventory", api_runtime_inventory, api_contract_inventory) or errors
    errors = compare_inventory("API error-code inventory", api_runtime_error_codes, api_contract_error_codes) or errors
    errors = compare_inventory("Realtime event inventory", realtime_runtime_events, realtime_contract_events) or errors
    errors = compare_inventory("Realtime error-code inventory", realtime_runtime_error_codes, realtime_contract_error_codes) or errors

    if engine.validate_api_semantic_contracts(API_CONTRACT) != 0:
        errors = True

    if errors:
        print("[contract-parity] Update runtime contract docs when API/realtime surface changes.")
        return 1

    print("[contract-parity] Runtime contract parity checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
