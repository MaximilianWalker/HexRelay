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

pub fn connection_ready_banner() -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: "realtime.connected".to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: Uuid::new_v4().to_string(),
        producer: "realtime-gateway".to_string(),
        data: serde_json::json!({ "state": "ok" }),
    };

    serde_json::to_string(&envelope).unwrap()
}
