use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
struct RealtimeInboundEnvelope {
    event_type: String,
    event_version: u8,
    #[serde(default)]
    correlation_id: Option<String>,
    data: Value,
}

#[derive(Deserialize, Serialize)]
struct CallSignalOfferData {
    call_id: String,
    from_user_id: String,
    to_user_id: String,
    sdp_offer: String,
}

#[derive(Deserialize, Serialize)]
struct CallSignalAnswerData {
    call_id: String,
    from_user_id: String,
    to_user_id: String,
    sdp_answer: String,
}

#[derive(Deserialize, Serialize)]
struct CallSignalIceCandidateData {
    call_id: String,
    from_user_id: String,
    to_user_id: String,
    candidate: String,
    #[serde(default)]
    sdp_mid: Option<String>,
    #[serde(default)]
    sdp_mline_index: Option<i32>,
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

#[derive(Serialize)]
struct RealtimeErrorData {
    code: String,
    message: String,
}

pub fn connection_ready_banner() -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: "realtime.connected".to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: Uuid::new_v4().to_string(),
        producer: "realtime-gateway".to_string(),
        data: serde_json::json!({ "status": "ok" }),
    };

    serde_json::to_string(&envelope)
        .unwrap_or_else(|_| "{\"event_type\":\"realtime.connected\"}".to_string())
}

pub fn route_inbound_event(raw: &str, session_identity_id: &str) -> String {
    let parsed = match serde_json::from_str::<RealtimeInboundEnvelope>(raw) {
        Ok(value) => value,
        Err(_) => {
            return build_error_event("event_invalid", "invalid event envelope payload");
        }
    };

    if parsed.event_version != 1 {
        return build_error_event("event_version_unsupported", "event_version must be 1");
    }

    match parsed.event_type.as_str() {
        "call.signal.offer" => match serde_json::from_value::<CallSignalOfferData>(parsed.data) {
            Ok(data) => {
                if data.from_user_id != session_identity_id {
                    return build_error_event(
                        "event_identity_mismatch",
                        "from_user_id does not match authenticated session",
                    );
                }

                if data.to_user_id != session_identity_id {
                    return build_error_event(
                        "event_unsupported",
                        "recipient-targeted signaling delivery not implemented",
                    );
                }

                build_event("call.signal.offer", parsed.correlation_id, data)
            }
            Err(_) => build_error_event("event_invalid", "invalid call.signal.offer payload"),
        },
        "call.signal.answer" => match serde_json::from_value::<CallSignalAnswerData>(parsed.data) {
            Ok(data) => {
                if data.from_user_id != session_identity_id {
                    return build_error_event(
                        "event_identity_mismatch",
                        "from_user_id does not match authenticated session",
                    );
                }

                if data.to_user_id != session_identity_id {
                    return build_error_event(
                        "event_unsupported",
                        "recipient-targeted signaling delivery not implemented",
                    );
                }

                build_event("call.signal.answer", parsed.correlation_id, data)
            }
            Err(_) => build_error_event("event_invalid", "invalid call.signal.answer payload"),
        },
        "call.signal.ice_candidate" => {
            match serde_json::from_value::<CallSignalIceCandidateData>(parsed.data) {
                Ok(data) => {
                    if data.from_user_id != session_identity_id {
                        return build_error_event(
                            "event_identity_mismatch",
                            "from_user_id does not match authenticated session",
                        );
                    }

                    if data.to_user_id != session_identity_id {
                        return build_error_event(
                            "event_unsupported",
                            "recipient-targeted signaling delivery not implemented",
                        );
                    }

                    build_event("call.signal.ice_candidate", parsed.correlation_id, data)
                }
                Err(_) => {
                    build_error_event("event_invalid", "invalid call.signal.ice_candidate payload")
                }
            }
        }
        _ => build_error_event("event_unsupported", "unsupported realtime event_type"),
    }
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

    serde_json::to_string(&envelope).unwrap_or_else(|_| {
        build_error_event("event_serialize_failed", "failed to serialize event")
    })
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

    serde_json::to_string(&envelope).unwrap_or_else(|_| "{\"event_type\":\"error\"}".to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_error_event, connection_ready_banner, route_inbound_event};
    use serde_json::Value;

    #[test]
    fn connection_ready_banner_has_expected_shape() {
        let banner = connection_ready_banner();
        let payload: Value = serde_json::from_str(&banner).expect("decode banner");

        assert_eq!(payload["event_type"], "realtime.connected");
        assert_eq!(payload["event_version"], 1);
        assert_eq!(payload["producer"], "realtime-gateway");
        assert_eq!(payload["data"]["status"], "ok");
    }

    #[test]
    fn routes_valid_answer_event() {
        let response = route_inbound_event(
            r#"{"event_type":"call.signal.answer","event_version":1,"correlation_id":"corr-1","data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-1","sdp_answer":"v=0\r\n"}}"#,
            "usr-1",
        );
        let payload: Value = serde_json::from_str(&response).expect("decode routed answer");

        assert_eq!(payload["event_type"], "call.signal.answer");
        assert_eq!(payload["correlation_id"], "corr-1");
        assert_eq!(payload["data"]["call_id"], "call-1");
    }

    #[test]
    fn routes_valid_ice_candidate_event() {
        let response = route_inbound_event(
            r#"{"event_type":"call.signal.ice_candidate","event_version":1,"data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-1","candidate":"candidate:1","sdp_mid":"0","sdp_mline_index":0}}"#,
            "usr-1",
        );
        let payload: Value = serde_json::from_str(&response).expect("decode routed candidate");

        assert_eq!(payload["event_type"], "call.signal.ice_candidate");
        assert_eq!(payload["data"]["candidate"], "candidate:1");
    }

    #[test]
    fn build_error_event_emits_error_envelope() {
        let response = build_error_event("event_invalid", "invalid payload");
        let payload: Value = serde_json::from_str(&response).expect("decode error");

        assert_eq!(payload["event_type"], "error");
        assert_eq!(payload["data"]["code"], "event_invalid");
        assert_eq!(payload["data"]["message"], "invalid payload");
    }

    #[test]
    fn rejects_cross_identity_recipient_targeting_until_delivery_exists() {
        let payloads = [
            r#"{"event_type":"call.signal.offer","event_version":1,"data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-2","sdp_offer":"v=0\r\n"}}"#,
            r#"{"event_type":"call.signal.answer","event_version":1,"data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-2","sdp_answer":"v=0\r\n"}}"#,
            r#"{"event_type":"call.signal.ice_candidate","event_version":1,"data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-2","candidate":"candidate:1"}}"#,
        ];

        for payload in payloads {
            let response = route_inbound_event(payload, "usr-1");
            let envelope: Value = serde_json::from_str(&response).expect("decode error envelope");
            assert_eq!(envelope["event_type"], "error");
            assert_eq!(envelope["data"]["code"], "event_unsupported");
        }
    }
}
