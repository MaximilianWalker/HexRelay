#!/usr/bin/env bash
set -euo pipefail

# Guard the DM trust boundary. Message nodes may store/fan out ciphertext
# envelopes, but runtime/config/contract surfaces must not introduce identifiers
# or settings for server plaintext access, private-key custody, or unencrypted
# mailbox/relay behavior.
runtime_forbidden_pattern='(dm[-_ ]?plain[-_ ]?text|plain[-_ ]?text[-_ ]?dm|clear[-_ ]?text[-_ ]?dm|dm[-_ ]?clear[-_ ]?text|server[-_ ]?readable[-_ ]?dm|dm[-_ ]?server[-_ ]?readable|decrypt[-_ ]?on[-_ ]?server|server[-_ ]?decrypt|server[-_ ]?side[-_ ]?decrypt(ion)?|dm[-_ ]?private[-_ ]?key|private[-_ ]?key[-_ ]?(upload|custody|escrow)|key[-_ ]?escrow|unencrypted[-_ ]?dm[-_ ]?(mailbox|relay|payload|storage)|dm[-_ ]?unencrypted[-_ ]?(mailbox|relay|payload|storage)|plain[-_ ]?text[-_ ]?relay|clear[-_ ]?text[-_ ]?relay|dm[-_ ]?plain[-_ ]?text[-_ ]?relay|dm[-_ ]?clear[-_ ]?text[-_ ]?relay)'
config_forbidden_pattern='(dm[-_ ]?plain[-_ ]?text|plain[-_ ]?text[-_ ]?dm|clear[-_ ]?text[-_ ]?dm|dm[-_ ]?clear[-_ ]?text|server[-_ ]?readable[-_ ]?dm|dm[-_ ]?server[-_ ]?readable|dm[-_ ]?decrypt[-_ ]?on[-_ ]?server|dm[-_ ]?server[-_ ]?decrypt|dm[-_ ]?server[-_ ]?side[-_ ]?decrypt(ion)?|dm[-_ ]?private[-_ ]?key|dm[-_ ]?private[-_ ]?key[-_ ]?(upload|custody|escrow)|dm[-_ ]?key[-_ ]?escrow|dm[-_ ]?unencrypted[-_ ]?(mailbox|relay|payload|storage)|unencrypted[-_ ]?dm[-_ ]?(mailbox|relay|payload|storage)|plain[-_ ]?text[-_ ]?relay|clear[-_ ]?text[-_ ]?relay|dm[-_ ]?plain[-_ ]?text[-_ ]?relay|dm[-_ ]?clear[-_ ]?text[-_ ]?relay)'
contract_forbidden_pattern='(dm_?plain_?text|plain_?text_?dm|clear_?text_?dm|dm_?clear_?text|server_?readable_?dm|dm_?server_?readable|dm_?decrypt_?on_?server|dm_?server_?decrypt|dm_?server_?side_?decrypt(ion)?|dm_?private_?key|dm_?private_?key_?(upload|custody|escrow)|dm_?key_?escrow|dm_?unencrypted_?(mailbox|relay|payload|storage)|unencrypted_?dm_?(mailbox|relay|payload|storage)|plain_?text_?relay|clear_?text_?relay|dm_?plain_?text_?relay|dm_?clear_?text_?relay)'
direct_dm_forbidden_pattern='(direct[-_ ]?only|direct[-_ ]?peer|DirectPeerTransport|dm[-_ ]?lan[-_ ]?discovery|dm\.lan_discovery|pairing[-_ ]?envelope|/dm/connectivity|endpoint[-_ ]?cards?|DmEndpointCard|wan[-_ ]?wizard|DmWanWizard|parallel[-_ ]?dial|DmParallelDial|DmConnectivityPreflight|dm_pairing|dm_lan_presence|dm_pairing_nonces)'
contact_qr_forbidden_pattern='(QRCodeSVG|qrcode\.react|IconQrcode|link \+ QR|QR code)'
dm_raw_log_metadata_pattern='(message_id|thread_id|recipient_identity_id|identity_id|device_id)[[:space:]]*=[[:space:]]*%'

matches=""

filter_allowed_direct_dm_matches() {
  grep -Ev '0011_dm_pairing_nonces|0014_dm_endpoint_cards_and_profile_devices|0019_remove_dm_direct_connect_tables' || true
}

core_matches="$(grep -RInEi "${runtime_forbidden_pattern}" --include='*.rs' "crates/communication-core/src" || true)"
if [ -n "${core_matches}" ]; then
  matches+="${core_matches}"$'\n'
fi

api_dm_matches="$(grep -RInEi "${runtime_forbidden_pattern}" --include='*.rs' "services/api-rs/src" || true)"
if [ -n "${api_dm_matches}" ]; then
  matches+="${api_dm_matches}"$'\n'
fi

realtime_matches="$(grep -RInEi "${runtime_forbidden_pattern}" --include='*.rs' "services/realtime-rs/src" || true)"
if [ -n "${realtime_matches}" ]; then
  matches+="${realtime_matches}"$'\n'
fi

config_matches="$(grep -InEi "${config_forbidden_pattern}" \
  ".github/workflows/ci.yml" \
  "docs/reference/runtime-config-reference.md" \
  "services/api-rs/src/config.rs" \
  "services/realtime-rs/src/config.rs" \
  || true)"
if [ -n "${config_matches}" ]; then
  matches+="${config_matches}"$'\n'
fi

contract_matches="$(grep -RInEi "${contract_forbidden_pattern}" \
  --include='*.yaml' \
  --include='*.yml' \
  "docs/contracts" \
  || true)"
if [ -n "${contract_matches}" ]; then
  matches+="${contract_matches}"$'\n'
fi

migration_matches="$(grep -RInEi "${contract_forbidden_pattern}" \
  --include='*.sql' \
  "services/api-rs/migrations" \
  || true)"
if [ -n "${migration_matches}" ]; then
  matches+="${migration_matches}"$'\n'
fi

fixture_matches="$(grep -RInEi "${contract_forbidden_pattern}" \
  --include='*.json' \
  --include='*.yaml' \
  --include='*.yml' \
  --include='*.rs' \
  --include='*.sql' \
  "scripts/fixtures" \
  || true)"
if [ -n "${fixture_matches}" ]; then
  matches+="${fixture_matches}"$'\n'
fi

evidence_matches="$(grep -RInEi "${contract_forbidden_pattern}" \
  --include='*.json' \
  --include='*.yaml' \
  --include='*.yml' \
  --include='*.md' \
  --include='*.txt' \
  "evidence" \
  || true)"
if [ -n "${evidence_matches}" ]; then
  matches+="${evidence_matches}"$'\n'
fi

direct_dm_runtime_matches="$(grep -RInE "${direct_dm_forbidden_pattern}" \
  --include='*.rs' \
  "crates/communication-core/src" \
  "services/api-rs/src" \
  "services/realtime-rs/src" \
  || true)"
direct_dm_runtime_matches="$(printf '%s' "${direct_dm_runtime_matches}" | filter_allowed_direct_dm_matches)"
if [ -n "${direct_dm_runtime_matches}" ]; then
  matches+="${direct_dm_runtime_matches}"$'\n'
fi

direct_dm_web_matches="$(grep -RInE "${direct_dm_forbidden_pattern}" \
  --include='*.ts' \
  --include='*.tsx' \
  "apps/web/app" \
  "apps/web/lib" \
  || true)"
if [ -n "${direct_dm_web_matches}" ]; then
  matches+="${direct_dm_web_matches}"$'\n'
fi

direct_dm_contract_matches="$(grep -RInE "${direct_dm_forbidden_pattern}" \
  --include='*.yaml' \
  --include='*.yml' \
  "docs/contracts" \
  || true)"
if [ -n "${direct_dm_contract_matches}" ]; then
  matches+="${direct_dm_contract_matches}"$'\n'
fi

direct_dm_fixture_matches="$(grep -RInE "${direct_dm_forbidden_pattern}" \
  --include='*.json' \
  --include='*.yaml' \
  --include='*.yml' \
  "scripts/fixtures" \
  || true)"
if [ -n "${direct_dm_fixture_matches}" ]; then
  matches+="${direct_dm_fixture_matches}"$'\n'
fi

contact_qr_matches="$(grep -RInE "${contact_qr_forbidden_pattern}" \
  --include='*.ts' \
  --include='*.tsx' \
  "apps/web/app/contacts" \
  || true)"
if [ -n "${contact_qr_matches}" ]; then
  matches+="${contact_qr_matches}"$'\n'
fi

dm_raw_log_metadata_matches="$(grep -RInE "${dm_raw_log_metadata_pattern}" \
  "services/api-rs/src/domain/dm/realtime.rs" \
  "services/realtime-rs/src/domain/dms.rs" \
  || true)"
if [ -n "${dm_raw_log_metadata_matches}" ]; then
  matches+="${dm_raw_log_metadata_matches}"$'\n'
fi

if [ -n "${matches}" ]; then
  echo "::error::Detected forbidden DM plaintext/key-custody terms, retired node-bypassing DM surfaces, contact-invite QR UI, or raw DM delivery metadata logs."
  echo "These terms are disallowed for DM E2EE envelope delivery policy:"
  printf '%s' "${matches}"
  exit 1
fi

echo "[dm-transport-policy] Runtime, config/workflow, web, and contract surfaces passed DM E2EE envelope policy guardrail"
