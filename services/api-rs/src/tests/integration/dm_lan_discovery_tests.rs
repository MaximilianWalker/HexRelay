use super::*;

#[tokio::test]
async fn announces_lan_presence_and_lists_policy_eligible_peer() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");

    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build lan peers request");

    let peers_response = app
        .clone()
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);

    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");

    let items = peers_payload["items"]
        .as_array()
        .expect("peers items array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["identity_id"], "usr-jules-p");
}

#[tokio::test]
async fn hides_lan_peer_when_peer_policy_disallows_sender() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");

    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build lan peers request");

    let peers_response = app
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);

    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");

    let items = peers_payload["items"]
        .as_array()
        .expect("peers items array");
    assert!(items.is_empty());
}

#[tokio::test]
async fn preflight_prefers_lan_fast_path_when_peer_is_discovered() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let policy_update = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_update)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");
    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let preflight_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p","local_bind_allowed":true,"peer_reachable_hint":true}"#,
        ))
        .expect("build preflight request");

    let preflight_response = app
        .oneshot(preflight_request)
        .await
        .expect("preflight response");
    assert_eq!(preflight_response.status(), StatusCode::OK);

    let body = to_bytes(preflight_response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "preflight_ok_lan");
}
