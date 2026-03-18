use super::*;

#[tokio::test]
async fn lists_servers_with_filters_from_persisted_memberships() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-nora-k"]).await else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-atlas-core",
        "Atlas Core",
        "usr-nora-k",
        true,
        false,
        2,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-relay-lab",
        "Relay Lab",
        "usr-nora-k",
        false,
        true,
        0,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-dev-signals",
        "Dev Signals",
        "usr-nora-k",
        true,
        false,
        5,
    )
    .await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/servers?favorites_only=true&unread_only=true")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
        )
        .body(Body::empty())
        .expect("build servers list request");

    let response = app.oneshot(request).await.expect("servers response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read servers response body");
    let payload: ServerListResponse =
        serde_json::from_slice(&body).expect("decode server list response");

    assert_eq!(payload.items.len(), 2);
    assert!(payload.items.iter().all(|item| item["favorite"] == true));
    assert!(payload
        .items
        .iter()
        .all(|item| item["unread"].as_u64().unwrap_or_default() > 0));
}

#[tokio::test]
async fn lists_servers_for_authenticated_identity_only() {
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&["usr-nora-k", "usr-alex-r"]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-atlas-core",
        "Atlas Core",
        "usr-nora-k",
        true,
        false,
        2,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-shared-lab",
        "Shared Lab",
        "usr-nora-k",
        false,
        false,
        1,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-shared-lab",
        "Shared Lab",
        "usr-alex-r",
        false,
        false,
        0,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-alex-craft",
        "Alex Craft",
        "usr-alex-r",
        true,
        false,
        1,
    )
    .await;

    let nora_request = Request::builder()
        .method("GET")
        .uri("/v1/servers")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
        )
        .body(Body::empty())
        .expect("build nora servers list request");
    let alex_request = Request::builder()
        .method("GET")
        .uri("/v1/servers")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-alex-r"]),
        )
        .body(Body::empty())
        .expect("build alex servers list request");

    let nora_response = app
        .clone()
        .oneshot(nora_request)
        .await
        .expect("nora servers response");
    let alex_response = app
        .clone()
        .oneshot(alex_request)
        .await
        .expect("alex servers response");
    assert_eq!(nora_response.status(), StatusCode::OK);
    assert_eq!(alex_response.status(), StatusCode::OK);

    let nora_body = to_bytes(nora_response.into_body(), usize::MAX)
        .await
        .expect("read nora servers body");
    let alex_body = to_bytes(alex_response.into_body(), usize::MAX)
        .await
        .expect("read alex servers body");
    let nora_payload: ServerListResponse =
        serde_json::from_slice(&nora_body).expect("decode nora server list response");
    let alex_payload: ServerListResponse =
        serde_json::from_slice(&alex_body).expect("decode alex server list response");

    assert!(nora_payload
        .items
        .iter()
        .any(|item| item["id"] == "srv-atlas-core"));
    assert!(nora_payload
        .items
        .iter()
        .any(|item| item["id"] == "srv-shared-lab"));
    assert!(nora_payload
        .items
        .iter()
        .all(|item| item["id"] != "srv-alex-craft"));

    assert!(alex_payload
        .items
        .iter()
        .any(|item| item["id"] == "srv-alex-craft"));
    assert!(alex_payload
        .items
        .iter()
        .any(|item| item["id"] == "srv-shared-lab"));
    assert!(alex_payload
        .items
        .iter()
        .all(|item| item["id"] != "srv-atlas-core"));
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
