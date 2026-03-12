use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

#[tokio::test]
async fn creates_and_imports_signed_dm_pairing_envelope() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["tcp://127.0.0.1:4040"],"expires_in_seconds":300}"#,
        ))
        .expect("build pairing envelope create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("pairing envelope create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope create body");
    let create_payload: serde_json::Value =
        serde_json::from_slice(&create_body).expect("decode pairing envelope create body");
    let envelope = create_payload["envelope"]
        .as_str()
        .expect("pairing envelope payload present");

    let import_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope/import")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"envelope":"{envelope}"}}"#)))
        .expect("build pairing envelope import request");

    let import_response = app
        .oneshot(import_request)
        .await
        .expect("pairing envelope import response");
    assert_eq!(import_response.status(), StatusCode::OK);

    let import_body = to_bytes(import_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope import body");
    let import_payload: serde_json::Value =
        serde_json::from_slice(&import_body).expect("decode pairing envelope import body");
    assert_eq!(import_payload["inviter_identity_id"], "usr-nora-k");
    assert_eq!(
        import_payload["endpoint_hints"][0],
        serde_json::Value::String("tcp://127.0.0.1:4040".to_string())
    );
}

#[tokio::test]
async fn rejects_replayed_pairing_envelope_nonce() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p", "usr-mina-s"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"endpoint_hints":["tcp://127.0.0.1:4040"]}"#))
        .expect("build pairing envelope create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("pairing envelope create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope create body");
    let create_payload: serde_json::Value =
        serde_json::from_slice(&create_body).expect("decode pairing envelope create body");
    let envelope = create_payload["envelope"]
        .as_str()
        .expect("pairing envelope payload present");

    let first_import = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope/import")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"envelope":"{envelope}"}}"#)))
        .expect("build first pairing import request");
    let first_response = app
        .clone()
        .oneshot(first_import)
        .await
        .expect("first pairing import response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_import = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope/import")
        .header("authorization", format!("Bearer {}", tokens["usr-mina-s"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"envelope":"{envelope}"}}"#)))
        .expect("build second pairing import request");

    let second_response = app
        .oneshot(second_import)
        .await
        .expect("second pairing import response");
    assert_eq!(second_response.status(), StatusCode::BAD_REQUEST);

    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("read second pairing import body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second pairing import body");
    assert_eq!(second_payload["code"], "pairing_replayed");
}

#[tokio::test]
async fn rejects_expired_pairing_envelope() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["tcp://127.0.0.1:4040"],"expires_in_seconds":1}"#,
        ))
        .expect("build pairing envelope create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("pairing envelope create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope create body");
    let create_payload: serde_json::Value =
        serde_json::from_slice(&create_body).expect("decode pairing envelope create body");
    let envelope = create_payload["envelope"]
        .as_str()
        .expect("pairing envelope payload present")
        .to_string();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let import_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope/import")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"envelope":"{envelope}"}}"#)))
        .expect("build pairing envelope import request");

    let import_response = app
        .oneshot(import_request)
        .await
        .expect("pairing envelope import response");
    assert_eq!(import_response.status(), StatusCode::BAD_REQUEST);

    let import_body = to_bytes(import_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope import body");
    let payload: serde_json::Value =
        serde_json::from_slice(&import_body).expect("decode pairing envelope import body");
    assert_eq!(payload["code"], "pairing_expired");
}

#[tokio::test]
async fn rejects_tampered_pairing_envelope_signature() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"endpoint_hints":["tcp://127.0.0.1:4040"]}"#))
        .expect("build pairing envelope create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("pairing envelope create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope create body");
    let create_payload: serde_json::Value =
        serde_json::from_slice(&create_body).expect("decode pairing envelope create body");
    let envelope = create_payload["envelope"]
        .as_str()
        .expect("pairing envelope payload present");

    let decoded = URL_SAFE_NO_PAD
        .decode(envelope)
        .expect("decode pairing envelope base64url");
    let mut signed: serde_json::Value =
        serde_json::from_slice(&decoded).expect("decode signed pairing envelope json");
    signed["claims"]["inviter_identity_id"] = serde_json::Value::String("usr-tamper".to_string());
    let tampered = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&signed).expect("encode tampered pairing envelope json"));

    let import_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope/import")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"envelope":"{tampered}"}}"#)))
        .expect("build pairing envelope import request");

    let import_response = app
        .oneshot(import_request)
        .await
        .expect("pairing envelope import response");
    assert_eq!(import_response.status(), StatusCode::BAD_REQUEST);

    let import_body = to_bytes(import_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope import body");
    let payload: serde_json::Value =
        serde_json::from_slice(&import_body).expect("decode pairing envelope import body");
    assert_eq!(payload["code"], "pairing_invalid");
}
