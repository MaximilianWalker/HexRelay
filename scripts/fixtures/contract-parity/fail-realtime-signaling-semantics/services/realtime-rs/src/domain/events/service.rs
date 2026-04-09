use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Deserialize, Serialize)]
struct CallSignalAnswerData {
    call_id: String,
    from_identity_id: String,
    to_identity_id: String,
    sdp_answer: String,
}

#[derive(Deserialize, Serialize)]
struct CallSignalIceCandidateData {
    call_id: String,
    from_identity_id: String,
    to_identity_id: String,
    candidate: String,
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
        "call.signal.answer" => match serde_json::from_value::<CallSignalAnswerData>(parsed.data) {
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

                build_event("call.signal.answer", parsed.correlation_id, data)
            }
            Err(_) => build_error_event("event_invalid", "invalid call.signal.answer payload"),
        },
        "call.signal.ice_candidate" => {
            match serde_json::from_value::<CallSignalIceCandidateData>(parsed.data) {
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

fn build_event<T: Serialize>(
    _event_type: &str,
    _correlation_id: Option<String>,
    _data: T,
) -> String {
    String::new()
}

fn build_error_event(_code: &str, _message: &str) -> String {
    String::new()
}
