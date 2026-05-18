use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::error::TrySendError;
use tracing::{info, warn};

use crate::state::{AppState, OutboundWsMessage};

const MAX_ID_LEN: usize = 128;
const MAX_IDENTITY_ID_LEN: usize = 64;
const MAX_DEVICE_ID_LEN: usize = 64;
const MIN_DEVICE_SECRET_LEN: usize = 16;
const MAX_DEVICE_SECRET_LEN: usize = 128;

#[derive(Clone)]
pub struct PublishDmEnvelopeInput {
    pub message_id: String,
    pub thread_id: String,
    pub sender_identity_id: String,
    pub recipient_identity_id: String,
    pub ciphertext: String,
    pub source_device_id: Option<String>,
    pub accepted_at: String,
    pub delivery_cursor: u64,
    pub target_device_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DmEnvelopeDispatchSummary {
    pub message_id: String,
    pub recipient_identity_id: String,
    pub target_device_count: u32,
    pub queued_device_ids: Vec<String>,
    pub pending_device_ids: Vec<String>,
    pub no_connection_device_ids: Vec<String>,
    pub unverified_device_ids: Vec<String>,
    pub saturated_device_ids: Vec<String>,
    pub stale_connection_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DmEnvelopeAckData {
    pub envelope_id: String,
    pub message_id: String,
    pub thread_id: String,
    pub recipient_identity_id: String,
    pub device_id: String,
    pub delivery_cursor: String,
    pub ack_status: String,
    pub received_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DmDeviceProofData {
    pub device_id: String,
    pub device_secret: String,
}

#[derive(Deserialize)]
struct RealtimeInboundEnvelope {
    event_type: String,
    #[serde(default)]
    correlation_id: Option<String>,
    data: serde_json::Value,
}

pub fn is_dm_envelope_ack_event(raw: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| {
            value
                .get("event_type")
                .and_then(|event_type| event_type.as_str())
                .map(|event_type| event_type == "dm.envelope.ack")
        })
        .unwrap_or(false)
}

pub fn is_dm_device_proof_event(raw: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| {
            value
                .get("event_type")
                .and_then(|event_type| event_type.as_str())
                .map(|event_type| event_type == "dm.device.proof")
        })
        .unwrap_or(false)
}

pub async fn publish_dm_envelope_dispatched(
    state: &AppState,
    input: PublishDmEnvelopeInput,
) -> Result<DmEnvelopeDispatchSummary, String> {
    validate_publish_input(&input)?;
    let target_device_ids = normalize_device_ids(&input.target_device_ids);
    if target_device_ids.is_empty() {
        return Ok(empty_dispatch_summary(
            &input.message_id,
            &input.recipient_identity_id,
        ));
    }

    let dispatched_at = Utc::now().to_rfc3339();
    let delivery_cursor = input.delivery_cursor.to_string();
    let events = target_device_ids
        .into_iter()
        .map(|target_device_id| {
            let envelope_id = dm_envelope_id(
                &input.message_id,
                &input.recipient_identity_id,
                &target_device_id,
                input.delivery_cursor,
            );
            let payload = crate::domain::events::service::build_dm_envelope_dispatched_event(
                &envelope_id,
                &input.message_id,
                &input.thread_id,
                &input.sender_identity_id,
                &input.recipient_identity_id,
                &target_device_id,
                &input.ciphertext,
                &input.accepted_at,
                &dispatched_at,
                &delivery_cursor,
                None,
            );
            (target_device_id, payload)
        })
        .collect::<Vec<_>>();

    let summary = dispatch_dm_envelopes_locally(
        state,
        &input.message_id,
        &input.recipient_identity_id,
        &events,
    )
    .await;
    info!(
        message_id = %summary.message_id,
        recipient_identity_id = %summary.recipient_identity_id,
        target_device_count = summary.target_device_count,
        queued_device_count = summary.queued_device_ids.len(),
        pending_device_count = summary.pending_device_ids.len(),
        no_connection_device_count = summary.no_connection_device_ids.len(),
        unverified_device_count = summary.unverified_device_ids.len(),
        saturated_device_count = summary.saturated_device_ids.len(),
        stale_connection_count = summary.stale_connection_count,
        "DM envelope realtime dispatch summarized"
    );
    Ok(summary)
}

pub async fn handle_dm_envelope_ack(
    state: &AppState,
    session_identity_id: &str,
    device_id: Option<&str>,
    dm_device_verified: bool,
    raw: &str,
) -> String {
    let (correlation_id, data) = match parse_dm_envelope_ack(raw, session_identity_id, device_id) {
        Ok(value) => value,
        Err(error) => {
            return crate::domain::events::service::build_error_event(error.code, error.message)
        }
    };
    if !dm_device_verified {
        return crate::domain::events::service::build_error_event(
            "event_device_mismatch",
            "verified websocket device binding is required for dm.envelope.ack",
        );
    }

    let url = format!(
        "{}/internal/dm/envelopes/ack",
        state.api_base_url.trim_end_matches('/')
    );
    match state
        .http_client
        .post(url)
        .header(
            "x-hexrelay-internal-token",
            state.channel_dispatch_internal_token.clone(),
        )
        .json(&data)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            crate::domain::events::service::build_dm_envelope_ack_event(
                &data.envelope_id,
                &data.message_id,
                &data.thread_id,
                &data.recipient_identity_id,
                &data.device_id,
                &data.delivery_cursor,
                &data.ack_status,
                &data.received_at,
                correlation_id,
            )
        }
        Ok(response) => {
            warn!(
                status = %response.status(),
                message_id = %data.message_id,
                recipient_identity_id = %data.recipient_identity_id,
                device_id = %data.device_id,
                "DM envelope ack upstream returned non-success status"
            );
            crate::domain::events::service::build_error_event(
                "dm_ack_failed",
                "failed to persist DM envelope ack",
            )
        }
        Err(error) => {
            warn!(
                error = %error,
                message_id = %data.message_id,
                recipient_identity_id = %data.recipient_identity_id,
                device_id = %data.device_id,
                "DM envelope ack upstream request failed"
            );
            crate::domain::events::service::build_error_event(
                "dm_ack_failed",
                "failed to persist DM envelope ack",
            )
        }
    }
}

pub async fn handle_dm_device_proof(
    state: &AppState,
    session_identity_id: &str,
    device_id: Option<&str>,
    raw: &str,
) -> (String, bool) {
    let (correlation_id, data) = match parse_dm_device_proof(raw, device_id) {
        Ok(value) => value,
        Err(error) => {
            return (
                crate::domain::events::service::build_error_event(error.code, error.message),
                false,
            )
        }
    };

    verify_dm_device_binding(
        state,
        session_identity_id,
        &data.device_id,
        &data.device_secret,
        correlation_id,
    )
    .await
}

pub async fn verify_dm_device_binding(
    state: &AppState,
    session_identity_id: &str,
    device_id: &str,
    device_secret: &str,
    correlation_id: Option<String>,
) -> (String, bool) {
    let url = format!(
        "{}/internal/dm/profile-devices/verify",
        state.api_base_url.trim_end_matches('/')
    );
    let response = state
        .http_client
        .post(url)
        .header(
            "x-hexrelay-internal-token",
            state.channel_dispatch_internal_token.clone(),
        )
        .json(&serde_json::json!({
            "identity_id": session_identity_id,
            "device_id": device_id,
            "device_secret": device_secret,
        }))
        .send()
        .await;

    match response {
        Ok(response) if response.status().is_success() => (
            crate::domain::events::service::build_dm_device_verified_event(
                device_id,
                correlation_id,
            ),
            true,
        ),
        Ok(response) => {
            warn!(
                status = %response.status(),
                identity_id = %session_identity_id,
                device_id = %device_id,
                "DM device proof upstream returned non-success status"
            );
            (
                crate::domain::events::service::build_error_event(
                    "event_device_mismatch",
                    "device proof did not match an active profile device",
                ),
                false,
            )
        }
        Err(error) => {
            warn!(
                error = %error,
                identity_id = %session_identity_id,
                device_id = %device_id,
                "DM device proof upstream request failed"
            );
            (
                crate::domain::events::service::build_error_event(
                    "dm_device_proof_failed",
                    "failed to verify DM device proof",
                ),
                false,
            )
        }
    }
}

async fn dispatch_dm_envelopes_locally(
    state: &AppState,
    message_id: &str,
    recipient_identity_id: &str,
    events: &[(String, String)],
) -> DmEnvelopeDispatchSummary {
    let mut device_states = events
        .iter()
        .map(|(device_id, _)| (device_id.clone(), DeviceDispatchState::NoConnection))
        .collect::<BTreeMap<_, _>>();
    let mut stale_connections = Vec::new();
    let mut guard = state.connection_senders.lock().await;

    if let Some(connections) = guard.get_mut(recipient_identity_id) {
        for (connection_id, entry) in connections.iter() {
            let Some(device_id) = entry.device_id.as_deref() else {
                continue;
            };
            let Some(device_state) = device_states.get_mut(device_id) else {
                continue;
            };
            if !entry.dm_device_verified {
                if *device_state == DeviceDispatchState::NoConnection {
                    *device_state = DeviceDispatchState::Unverified;
                }
                continue;
            }

            let Some((_, payload)) = events
                .iter()
                .find(|(target_device_id, _)| target_device_id == device_id)
            else {
                continue;
            };
            match entry
                .sender
                .try_send(OutboundWsMessage::text(payload.clone()))
            {
                Ok(()) => {
                    *device_state = DeviceDispatchState::Queued;
                }
                Err(TrySendError::Closed(_)) => {
                    stale_connections.push(connection_id.clone());
                }
                Err(TrySendError::Full(_)) => {
                    if *device_state != DeviceDispatchState::Queued {
                        *device_state = DeviceDispatchState::Saturated;
                    }
                    warn!(
                        recipient_identity_id = %recipient_identity_id,
                        connection_id = %connection_id,
                        device_id = %device_id,
                        "DM envelope outbound queue saturated; keeping websocket registered"
                    );
                }
            }
        }

        for connection_id in &stale_connections {
            connections.remove(connection_id);
        }
        if connections.is_empty() {
            guard.remove(recipient_identity_id);
        }
    }

    build_dispatch_summary(
        message_id,
        recipient_identity_id,
        device_states,
        stale_connections.len(),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeviceDispatchState {
    NoConnection,
    Unverified,
    Saturated,
    Queued,
}

fn empty_dispatch_summary(
    message_id: &str,
    recipient_identity_id: &str,
) -> DmEnvelopeDispatchSummary {
    DmEnvelopeDispatchSummary {
        message_id: message_id.to_string(),
        recipient_identity_id: recipient_identity_id.to_string(),
        target_device_count: 0,
        queued_device_ids: Vec::new(),
        pending_device_ids: Vec::new(),
        no_connection_device_ids: Vec::new(),
        unverified_device_ids: Vec::new(),
        saturated_device_ids: Vec::new(),
        stale_connection_count: 0,
    }
}

fn build_dispatch_summary(
    message_id: &str,
    recipient_identity_id: &str,
    device_states: BTreeMap<String, DeviceDispatchState>,
    stale_connection_count: usize,
) -> DmEnvelopeDispatchSummary {
    let mut queued_device_ids = Vec::new();
    let mut pending_device_ids = Vec::new();
    let mut no_connection_device_ids = Vec::new();
    let mut unverified_device_ids = Vec::new();
    let mut saturated_device_ids = Vec::new();

    for (device_id, state) in &device_states {
        match state {
            DeviceDispatchState::Queued => queued_device_ids.push(device_id.clone()),
            DeviceDispatchState::NoConnection => {
                pending_device_ids.push(device_id.clone());
                no_connection_device_ids.push(device_id.clone());
            }
            DeviceDispatchState::Unverified => {
                pending_device_ids.push(device_id.clone());
                unverified_device_ids.push(device_id.clone());
            }
            DeviceDispatchState::Saturated => {
                pending_device_ids.push(device_id.clone());
                saturated_device_ids.push(device_id.clone());
            }
        }
    }

    DmEnvelopeDispatchSummary {
        message_id: message_id.to_string(),
        recipient_identity_id: recipient_identity_id.to_string(),
        target_device_count: u32::try_from(device_states.len()).unwrap_or(u32::MAX),
        queued_device_ids,
        pending_device_ids,
        no_connection_device_ids,
        unverified_device_ids,
        saturated_device_ids,
        stale_connection_count: u32::try_from(stale_connection_count).unwrap_or(u32::MAX),
    }
}

fn validate_publish_input(input: &PublishDmEnvelopeInput) -> Result<(), String> {
    validate_required_id(&input.message_id, "message_id", MAX_ID_LEN)?;
    validate_required_id(&input.thread_id, "thread_id", MAX_ID_LEN)?;
    validate_required_id(
        &input.sender_identity_id,
        "sender_identity_id",
        MAX_IDENTITY_ID_LEN,
    )?;
    validate_required_id(
        &input.recipient_identity_id,
        "recipient_identity_id",
        MAX_IDENTITY_ID_LEN,
    )?;
    if input.ciphertext.trim().is_empty() {
        return Err("ciphertext must not be empty".to_string());
    }
    DateTime::parse_from_rfc3339(&input.accepted_at)
        .map_err(|_| "accepted_at must be an RFC3339 date-time".to_string())?;
    if input.delivery_cursor == 0 {
        return Err("delivery_cursor must be greater than zero".to_string());
    }
    if let Some(source_device_id) = &input.source_device_id {
        validate_required_id(source_device_id, "source_device_id", MAX_DEVICE_ID_LEN)?;
    }
    for target_device_id in &input.target_device_ids {
        validate_required_id(target_device_id, "target_device_id", MAX_DEVICE_ID_LEN)?;
    }

    Ok(())
}

fn parse_dm_envelope_ack(
    raw: &str,
    session_identity_id: &str,
    session_device_id: Option<&str>,
) -> Result<(Option<String>, DmEnvelopeAckData), DmAckError> {
    let envelope =
        serde_json::from_str::<RealtimeInboundEnvelope>(raw).map_err(|_| DmAckError {
            code: "event_invalid",
            message: "invalid event envelope payload",
        })?;
    if envelope.event_type != "dm.envelope.ack" {
        return Err(DmAckError {
            code: "event_unsupported",
            message: "unsupported realtime event_type",
        });
    }
    let data =
        serde_json::from_value::<DmEnvelopeAckData>(envelope.data).map_err(|_| DmAckError {
            code: "event_invalid",
            message: "invalid dm.envelope.ack payload",
        })?;

    validate_ack_data(&data)?;
    if data.recipient_identity_id != session_identity_id {
        return Err(DmAckError {
            code: "event_identity_mismatch",
            message: "recipient_identity_id does not match authenticated session",
        });
    }
    if Some(data.device_id.as_str()) != session_device_id {
        return Err(DmAckError {
            code: "event_device_mismatch",
            message: "device_id does not match authenticated websocket device",
        });
    }

    Ok((envelope.correlation_id, data))
}

fn parse_dm_device_proof(
    raw: &str,
    session_device_id: Option<&str>,
) -> Result<(Option<String>, DmDeviceProofData), DmAckError> {
    let envelope =
        serde_json::from_str::<RealtimeInboundEnvelope>(raw).map_err(|_| DmAckError {
            code: "event_invalid",
            message: "invalid event envelope payload",
        })?;
    if envelope.event_type != "dm.device.proof" {
        return Err(DmAckError {
            code: "event_unsupported",
            message: "unsupported realtime event_type",
        });
    }

    let data =
        serde_json::from_value::<DmDeviceProofData>(envelope.data).map_err(|_| DmAckError {
            code: "event_invalid",
            message: "invalid dm.device.proof payload",
        })?;
    validate_required_id(&data.device_id, "device_id", MAX_DEVICE_ID_LEN).map_err(|_| {
        DmAckError {
            code: "event_invalid",
            message: "invalid dm.device.proof payload",
        }
    })?;
    validate_device_secret(&data.device_secret).map_err(|_| DmAckError {
        code: "event_invalid",
        message: "invalid dm.device.proof payload",
    })?;
    if Some(data.device_id.as_str()) != session_device_id {
        return Err(DmAckError {
            code: "event_device_mismatch",
            message: "device_id does not match authenticated websocket device",
        });
    }

    Ok((envelope.correlation_id, data))
}

fn validate_ack_data(data: &DmEnvelopeAckData) -> Result<(), DmAckError> {
    validate_required_id(&data.envelope_id, "envelope_id", MAX_ID_LEN).map_err(|_| DmAckError {
        code: "event_invalid",
        message: "invalid dm.envelope.ack payload",
    })?;
    validate_required_id(&data.message_id, "message_id", MAX_ID_LEN).map_err(|_| DmAckError {
        code: "event_invalid",
        message: "invalid dm.envelope.ack payload",
    })?;
    validate_required_id(&data.thread_id, "thread_id", MAX_ID_LEN).map_err(|_| DmAckError {
        code: "event_invalid",
        message: "invalid dm.envelope.ack payload",
    })?;
    validate_required_id(
        &data.recipient_identity_id,
        "recipient_identity_id",
        MAX_IDENTITY_ID_LEN,
    )
    .map_err(|_| DmAckError {
        code: "event_invalid",
        message: "invalid dm.envelope.ack payload",
    })?;
    validate_required_id(&data.device_id, "device_id", MAX_DEVICE_ID_LEN).map_err(|_| {
        DmAckError {
            code: "event_invalid",
            message: "invalid dm.envelope.ack payload",
        }
    })?;
    if data.ack_status != "received" {
        return Err(DmAckError {
            code: "event_invalid",
            message: "ack_status must be received",
        });
    }
    let cursor = data
        .delivery_cursor
        .trim()
        .parse::<u64>()
        .map_err(|_| DmAckError {
            code: "event_invalid",
            message: "delivery_cursor must be numeric",
        })?;
    if cursor == 0 || data.delivery_cursor.trim() != data.delivery_cursor {
        return Err(DmAckError {
            code: "event_invalid",
            message: "delivery_cursor must be a positive numeric string",
        });
    }
    DateTime::parse_from_rfc3339(data.received_at.trim()).map_err(|_| DmAckError {
        code: "event_invalid",
        message: "received_at must be an RFC3339 date-time",
    })?;
    if data.received_at.trim() != data.received_at {
        return Err(DmAckError {
            code: "event_invalid",
            message: "received_at must not include leading or trailing whitespace",
        });
    }

    Ok(())
}

fn validate_required_id(value: &str, name: &str, max_len: usize) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > max_len || trimmed != value {
        return Err(format!(
            "{name} must be non-empty, trimmed, and <= {max_len} chars"
        ));
    }
    Ok(())
}

fn validate_device_secret(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.len() < MIN_DEVICE_SECRET_LEN || trimmed.len() > MAX_DEVICE_SECRET_LEN {
        return Err("device_secret must be 16-128 chars".to_string());
    }
    if trimmed != value {
        return Err("device_secret must not include leading or trailing whitespace".to_string());
    }
    if !trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("device_secret must use only letters, numbers, _ or -".to_string());
    }

    Ok(())
}

fn normalize_device_ids(device_ids: &[String]) -> Vec<String> {
    let mut deduped = BTreeSet::new();
    for device_id in device_ids {
        let trimmed = device_id.trim();
        if !trimmed.is_empty() {
            deduped.insert(trimmed.to_string());
        }
    }
    deduped.into_iter().collect()
}

fn dm_envelope_id(
    message_id: &str,
    recipient_identity_id: &str,
    target_device_id: &str,
    delivery_cursor: u64,
) -> String {
    let material =
        format!("{message_id}:{recipient_identity_id}:{target_device_id}:{delivery_cursor}");
    let digest = Sha256::digest(material.as_bytes());
    format!("dm-env-{}", lower_hex(&digest))
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[derive(Debug)]
struct DmAckError {
    code: &'static str,
    message: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;
    use crate::state::OutboundWsMessage;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn dispatch_dm_envelope_sends_only_to_target_devices() {
        let state = test_state();
        let (desktop_tx, mut desktop_rx) = mpsc::channel::<OutboundWsMessage>(1);
        let (phone_tx, mut phone_rx) = mpsc::channel::<OutboundWsMessage>(1);
        let (other_tx, mut other_rx) = mpsc::channel::<OutboundWsMessage>(1);

        state.connection_senders.lock().await.insert(
            "usr-recipient".to_string(),
            std::collections::HashMap::from([
                (
                    "conn-desktop".to_string(),
                    crate::state::ConnectionSenderEntry {
                        sender: desktop_tx,
                        device_id: Some("desktop-main".to_string()),
                        dm_device_verified: true,
                    },
                ),
                (
                    "conn-phone".to_string(),
                    crate::state::ConnectionSenderEntry {
                        sender: phone_tx,
                        device_id: Some("phone-main".to_string()),
                        dm_device_verified: true,
                    },
                ),
                (
                    "conn-other".to_string(),
                    crate::state::ConnectionSenderEntry {
                        sender: other_tx,
                        device_id: Some("tablet-main".to_string()),
                        dm_device_verified: true,
                    },
                ),
            ]),
        );

        let summary = publish_dm_envelope_dispatched(
            &state,
            PublishDmEnvelopeInput {
                message_id: "msg-1".to_string(),
                thread_id: "thread-1".to_string(),
                sender_identity_id: "usr-sender".to_string(),
                recipient_identity_id: "usr-recipient".to_string(),
                ciphertext: "enc:abcdefghijklmnopqrstuvwxyz".to_string(),
                source_device_id: Some("sender-main".to_string()),
                accepted_at: "2026-03-26T00:00:00Z".to_string(),
                delivery_cursor: 3,
                target_device_ids: vec!["desktop-main".to_string(), "phone-main".to_string()],
            },
        )
        .await
        .expect("publish DM envelope");
        assert_eq!(summary.target_device_count, 2);
        assert_eq!(
            summary.queued_device_ids,
            vec!["desktop-main".to_string(), "phone-main".to_string()]
        );
        assert!(summary.pending_device_ids.is_empty());
        assert!(summary.no_connection_device_ids.is_empty());

        let desktop_payload = desktop_rx.recv().await.expect("desktop event");
        let phone_payload = phone_rx.recv().await.expect("phone event");
        assert!(other_rx.try_recv().is_err());

        for (payload, target_device_id) in [
            (desktop_payload, "desktop-main"),
            (phone_payload, "phone-main"),
        ] {
            let event: serde_json::Value = serde_json::from_str(&payload).expect("decode event");
            assert_eq!(event["event_type"], "dm.envelope.dispatched");
            assert_eq!(event["producer"], "dm-message-node");
            assert_eq!(event["data"]["message_id"], "msg-1");
            assert_eq!(event["data"]["thread_id"], "thread-1");
            assert_eq!(event["data"]["recipient_identity_id"], "usr-recipient");
            assert_eq!(event["data"]["target_device_id"], target_device_id);
            assert_eq!(event["data"]["delivery_cursor"], "3");
            assert_eq!(event["data"]["transport_scope"], "encrypted_envelope_node");
        }
    }

    #[tokio::test]
    async fn dispatch_dm_envelope_keeps_full_connections_registered() {
        let state = test_state();
        let (full_tx, mut full_rx) = mpsc::channel::<OutboundWsMessage>(1);
        full_tx
            .try_send(OutboundWsMessage::text("seed"))
            .expect("fill outbound queue");

        state.connection_senders.lock().await.insert(
            "usr-recipient".to_string(),
            std::collections::HashMap::from([(
                "conn-full".to_string(),
                crate::state::ConnectionSenderEntry {
                    sender: full_tx,
                    device_id: Some("desktop-main".to_string()),
                    dm_device_verified: true,
                },
            )]),
        );

        let summary = publish_dm_envelope_dispatched(
            &state,
            PublishDmEnvelopeInput {
                message_id: "msg-1".to_string(),
                thread_id: "thread-1".to_string(),
                sender_identity_id: "usr-sender".to_string(),
                recipient_identity_id: "usr-recipient".to_string(),
                ciphertext: "enc:abcdefghijklmnopqrstuvwxyz".to_string(),
                source_device_id: None,
                accepted_at: "2026-03-26T00:00:00Z".to_string(),
                delivery_cursor: 3,
                target_device_ids: vec!["desktop-main".to_string()],
            },
        )
        .await
        .expect("publish DM envelope");
        assert_eq!(summary.target_device_count, 1);
        assert!(summary.queued_device_ids.is_empty());
        assert_eq!(summary.pending_device_ids, vec!["desktop-main".to_string()]);
        assert_eq!(
            summary.saturated_device_ids,
            vec!["desktop-main".to_string()]
        );

        assert_eq!(full_rx.recv().await.as_deref(), Some("seed"));
        let guard = state.connection_senders.lock().await;
        let connections = guard.get("usr-recipient").expect("remaining connections");
        assert!(connections.contains_key("conn-full"));
    }

    #[tokio::test]
    async fn dispatch_dm_envelope_reports_pending_target_reasons() {
        let state = test_state();
        let (unverified_tx, mut unverified_rx) = mpsc::channel::<OutboundWsMessage>(1);
        let (closed_tx, closed_rx) = mpsc::channel::<OutboundWsMessage>(1);
        drop(closed_rx);

        state.connection_senders.lock().await.insert(
            "usr-recipient".to_string(),
            std::collections::HashMap::from([
                (
                    "conn-unverified".to_string(),
                    crate::state::ConnectionSenderEntry {
                        sender: unverified_tx,
                        device_id: Some("phone-main".to_string()),
                        dm_device_verified: false,
                    },
                ),
                (
                    "conn-closed".to_string(),
                    crate::state::ConnectionSenderEntry {
                        sender: closed_tx,
                        device_id: Some("desktop-main".to_string()),
                        dm_device_verified: true,
                    },
                ),
            ]),
        );

        let summary = publish_dm_envelope_dispatched(
            &state,
            PublishDmEnvelopeInput {
                message_id: "msg-1".to_string(),
                thread_id: "thread-1".to_string(),
                sender_identity_id: "usr-sender".to_string(),
                recipient_identity_id: "usr-recipient".to_string(),
                ciphertext: "enc:abcdefghijklmnopqrstuvwxyz".to_string(),
                source_device_id: None,
                accepted_at: "2026-03-26T00:00:00Z".to_string(),
                delivery_cursor: 3,
                target_device_ids: vec![
                    "desktop-main".to_string(),
                    "phone-main".to_string(),
                    "tablet-main".to_string(),
                ],
            },
        )
        .await
        .expect("publish DM envelope");

        assert_eq!(summary.target_device_count, 3);
        assert!(summary.queued_device_ids.is_empty());
        assert_eq!(
            summary.pending_device_ids,
            vec![
                "desktop-main".to_string(),
                "phone-main".to_string(),
                "tablet-main".to_string()
            ]
        );
        assert_eq!(
            summary.no_connection_device_ids,
            vec!["desktop-main".to_string(), "tablet-main".to_string()]
        );
        assert_eq!(
            summary.unverified_device_ids,
            vec!["phone-main".to_string()]
        );
        assert_eq!(summary.stale_connection_count, 1);
        assert!(unverified_rx.try_recv().is_err());
        let guard = state.connection_senders.lock().await;
        let connections = guard.get("usr-recipient").expect("remaining connection");
        assert!(!connections.contains_key("conn-closed"));
        assert!(connections.contains_key("conn-unverified"));
    }

    #[test]
    fn dm_ack_requires_matching_session_identity_and_device() {
        let raw = r#"{"event_type":"dm.envelope.ack","data":{"envelope_id":"dm-env-12345678","message_id":"msg-1","thread_id":"thread-1","recipient_identity_id":"usr-recipient","device_id":"desktop-main","delivery_cursor":"3","ack_status":"received","received_at":"2026-03-26T00:00:01Z"}}"#;

        assert!(parse_dm_envelope_ack(raw, "usr-recipient", Some("desktop-main")).is_ok());
        let wrong_identity =
            parse_dm_envelope_ack(raw, "usr-other", Some("desktop-main")).unwrap_err();
        assert_eq!(wrong_identity.code, "event_identity_mismatch");
        let wrong_device =
            parse_dm_envelope_ack(raw, "usr-recipient", Some("phone-main")).unwrap_err();
        assert_eq!(wrong_device.code, "event_device_mismatch");
    }

    fn test_state() -> AppState {
        AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
            "hexrelay-dev-presence-watcher-token-change-me".to_string(),
            None,
            false,
            60,
            60,
            32 * 1024,
            120,
            60,
            3,
            5,
            2048,
        )
        .expect("build app state")
    }
}
