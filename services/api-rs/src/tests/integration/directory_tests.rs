use super::*;

#[tokio::test]
async fn lists_servers_with_filters() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/servers?favorites_only=true&unread_only=true")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build servers list request");

    let response = app.oneshot(request).await.expect("servers response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read servers response body");
    let payload: ServerListResponse =
        serde_json::from_slice(&body).expect("decode server list response");
    assert!(!payload.items.is_empty());
}

#[tokio::test]
async fn lists_contacts_with_search_filter() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/contacts?search=nora")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build contacts list request");

    let response = app.oneshot(request).await.expect("contacts response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read contacts response body");
    let payload: ContactListResponse =
        serde_json::from_slice(&body).expect("decode contacts list response");
    assert_eq!(payload.items.len(), 1);
}
