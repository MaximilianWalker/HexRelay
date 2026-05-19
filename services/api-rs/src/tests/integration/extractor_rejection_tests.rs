use super::*;

async fn assert_api_error(
    response: axum::response::Response,
    status: StatusCode,
    code: &str,
    message: &str,
) {
    assert_eq!(response.status(), status);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(
        content_type.starts_with("application/json"),
        "expected JSON ApiError response, got content-type {content_type:?}"
    );

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read extractor rejection body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode extractor rejection body");
    assert_eq!(payload["code"], code);
    assert_eq!(payload["message"], message);
}

#[tokio::test]
async fn malformed_json_uses_api_error_shape() {
    let app = build_app(test_state_with_public_identity_registration());
    let request = Request::builder()
        .method("POST")
        .uri("/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"usr-extractor""#))
        .expect("build malformed JSON request");

    let response = app.oneshot(request).await.expect("malformed JSON response");

    assert_api_error(
        response,
        StatusCode::BAD_REQUEST,
        "request_body_invalid",
        "request body must be valid JSON",
    )
    .await;
}

#[tokio::test]
async fn unsupported_json_content_type_uses_api_error_shape() {
    let app = build_app(test_state_with_public_identity_registration());
    let request = Request::builder()
        .method("POST")
        .uri("/auth/challenge")
        .header("content-type", "text/plain")
        .body(Body::from(r#"{"identity_id":"usr-extractor"}"#))
        .expect("build unsupported content-type request");

    let response = app
        .oneshot(request)
        .await
        .expect("unsupported content-type response");

    assert_api_error(
        response,
        StatusCode::BAD_REQUEST,
        "content_type_unsupported",
        "request content type must be application/json",
    )
    .await;
}

#[tokio::test]
async fn invalid_query_uses_api_error_shape() {
    let (app, tokens) = app_with_sessions(&["usr-extractor-query"]);
    let request = Request::builder()
        .method("GET")
        .uri("/servers?favorites_only=definitely-not-a-bool")
        .header(
            "authorization",
            format!("Bearer {}", tokens["usr-extractor-query"]),
        )
        .body(Body::empty())
        .expect("build invalid query request");

    let response = app.oneshot(request).await.expect("invalid query response");

    assert_api_error(
        response,
        StatusCode::BAD_REQUEST,
        "query_invalid",
        "query parameters are invalid",
    )
    .await;
}
