use super::*;

#[tokio::test]
async fn wan_wizard_reports_success_with_upnp_mapping() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/wan-wizard")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"preferred_port":4040,"upnp_available":true,"auto_mapping_succeeds":true,"network_profile":"home_nat"}"#,
        ))
        .expect("build wan wizard request");

    let response = app.oneshot(request).await.expect("wan wizard response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read wan wizard body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode wan wizard response");
    assert_eq!(payload["outcome"], "success");
    assert_eq!(payload["method"], "upnp");
    assert_eq!(payload["reason_code"], "wan_path_ready");
}

#[tokio::test]
async fn wan_wizard_reports_manual_required_when_auto_mapping_fails() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/wan-wizard")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"preferred_port":4040,"upnp_available":true,"auto_mapping_succeeds":false,"network_profile":"home_nat"}"#,
        ))
        .expect("build wan wizard request");

    let response = app.oneshot(request).await.expect("wan wizard response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read wan wizard body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode wan wizard response");
    assert_eq!(payload["outcome"], "manual_required");
    assert_eq!(payload["method"], "manual");
    assert_eq!(payload["reason_code"], "wan_manual_mapping_required");
}

#[tokio::test]
async fn wan_wizard_reports_network_incompatible_for_restricted_profiles() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/wan-wizard")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"preferred_port":4040,"network_profile":"carrier_nat","upnp_available":false,"nat_pmp_available":false,"auto_mapping_succeeds":false,"external_port_open":false}"#,
        ))
        .expect("build wan wizard request");

    let response = app.oneshot(request).await.expect("wan wizard response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read wan wizard body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode wan wizard response");
    assert_eq!(payload["outcome"], "network_incompatible");
    assert_eq!(payload["method"], "none");
    assert_eq!(payload["reason_code"], "wan_path_unavailable");
}
