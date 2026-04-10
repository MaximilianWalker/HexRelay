use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
struct RealtimeInboundEnvelope {
    event_type: String,
    event_version: u8,
    correlation_id: Option<String>,
    data: Value,
}

#[derive(Deserialize, Serialize)]
struct CallSignalOfferData {
    call_id: String,
    from_identity_id: String,
    to_identity_id: String,
    sdp_offer: String,
}

#[derive(Serialize)]
struct RealtimeOutboundEnvelope<T: Serialize> {
    event_id: String,
    event_type: String,
    event_version: u8,
    occurred_at: String,
    correlation_id: String,
    producer: String,
    data: T,
}

fn build_event<T: Serialize>(event_type: &str, correlation_id: Option<String>, data: T) -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: event_type.to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: correlation_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        producer: "realtime-gateway".to_string(),
        data,
    };

    serde_json::to_string(&envelope).unwrap()
}

fn build_error_event(_code: &str, _message: &str) -> String {
    String::new()
}

pub fn route_inbound_event(raw: &str, session_identity_id: &str) -> String {
    let parsed = match serde_json::from_str::<RealtimeInboundEnvelope>(raw) {
        Ok(value) => value,
        Err(_) => return build_error_event("event_invalid", "invalid event envelope payload"),
    };

    if parsed.event_version != 1 {
        return build_error_event("event_version_unsupported", "event_version must be 1");
    }

    match parsed.event_type.as_str() {
        "call.signal.offer" => match serde_json::from_value::<CallSignalOfferData>(parsed.data) {
            Ok(data) => {
                if data.from_identity_id != session_identity_id {
                    return build_error_event(
                        "event_identity_mismatch",
                        "from_identity_id does not match authenticated session",
                    );
                }

                if data.to_identity_id != session_identity_id {
                    return build_error_event(
                        "event_unsupported",
                        "recipient-targeted signaling delivery not implemented",
                    );
                }

                build_event("call.signal.offer", parsed.correlation_id, data)
            }
            Err(_) => build_error_event("event_invalid", "invalid call.signal.offer payload"),
        },
        _ => build_error_event("event_unsupported", "unsupported realtime event_type"),
    }
}
