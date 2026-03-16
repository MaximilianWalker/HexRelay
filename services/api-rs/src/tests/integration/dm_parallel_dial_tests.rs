use super::*;

#[tokio::test]
async fn parallel_dial_selects_fastest_reachable_endpoint_and_cancels_others() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"wan-a","endpoint_hint":"udp://203.0.113.20:4040","estimated_rtt_ms":70,"priority":1},{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":9,"priority":2},{"endpoint_id":"wan-b","endpoint_hint":"tcp://198.51.100.20:4041","estimated_rtt_ms":40,"priority":1}]}"#,
        ))
        .expect("build endpoint register request");

    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"peer_identity_id":"usr-jules-p","max_parallel_attempts":3}"#,
        ))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "parallel_dial_connected");
    assert_eq!(payload["winner_endpoint_id"], "lan-a");

    let canceled = payload["canceled_endpoint_ids"]
        .as_array()
        .expect("canceled endpoints array");
    assert_eq!(canceled.len(), 2);
}

#[tokio::test]
async fn parallel_dial_ignores_revoked_endpoint_cards() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":10},{"endpoint_id":"wan-a","endpoint_hint":"tcp://198.51.100.20:4041","estimated_rtt_ms":30}]}"#,
        ))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards/revoke")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"endpoint_ids":["lan-a"]}"#))
        .expect("build endpoint revoke request");
    let revoke_response = app
        .clone()
        .oneshot(revoke_request)
        .await
        .expect("endpoint revoke response");
    assert_eq!(revoke_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"peer_identity_id":"usr-jules-p","max_parallel_attempts":2}"#,
        ))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["winner_endpoint_id"], "wan-a");
}

#[tokio::test]
async fn parallel_dial_returns_blocked_when_all_attempts_fail() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"wan-a","endpoint_hint":"udp://203.0.113.20:4040","estimated_rtt_ms":70},{"endpoint_id":"wan-b","endpoint_hint":"tcp://198.51.100.20:4041","estimated_rtt_ms":40}]}"#,
        ))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"peer_identity_id":"usr-jules-p","unreachable_endpoint_ids":["wan-a","wan-b"]}"#,
        ))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "parallel_dial_exhausted");
}
