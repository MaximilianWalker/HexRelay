use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

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

#[derive(Serialize)]
struct RealtimeErrorData {
    code: String,
    message: String,
}

pub(crate) fn build_error_event(code: &str, message: &str) -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: "error".to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: Uuid::new_v4().to_string(),
        producer: "realtime-gateway".to_string(),
        data: RealtimeErrorData {
            code: code.to_string(),
            message: message.to_string(),
        },
    };

    serde_json::to_string(&envelope).unwrap()
}
