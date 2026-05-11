use super::*;

#[tokio::test]
async fn node_connection_returns_public_app_connection_contract() {
    let app = build_app(AppState::default());

    let request = Request::builder()
        .method("GET")
        .uri("/node/connection")
        .body(Body::empty())
        .expect("build node connection request");

    let response = app
        .oneshot(request)
        .await
        .expect("node connection response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read node connection body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode node connection body");

    assert_eq!(payload["service"], "api-rs");
    assert_eq!(payload["node_id"], TEST_NODE_FINGERPRINT);
    assert_eq!(payload["node_fingerprint"], TEST_NODE_FINGERPRINT);
    assert_eq!(payload["runtime_api"], "hexrelay_runtime_rest");
    assert_eq!(payload["auth_endpoints"]["challenge"], "/auth/challenge");
    assert_eq!(payload["auth_endpoints"]["verify"], "/auth/verify");
    assert_eq!(
        payload["auth_endpoints"]["session_validate"],
        "/auth/sessions/validate"
    );
    assert_eq!(payload["capabilities_endpoint"], "/node/capabilities");
    assert!(payload.get("administration").is_none());
}

#[tokio::test]
async fn node_capabilities_require_authenticated_identity() {
    let app = build_app(AppState::default());

    let request = Request::builder()
        .method("GET")
        .uri("/node/capabilities")
        .body(Body::empty())
        .expect("build node capabilities request");

    let response = app
        .oneshot(request)
        .await
        .expect("node capabilities response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read node capabilities error body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode node capabilities error body");
    assert_eq!(payload["code"], "session_invalid");
}

#[tokio::test]
async fn node_capabilities_report_configured_owner_admin_and_member_permissions() {
    let (app, tokens) = app_with_node_scope_sessions(
        &["usr-owner"],
        &["usr-admin"],
        &["usr-owner", "usr-admin", "usr-member"],
    );

    let owner = get_capabilities(app.clone(), &tokens["usr-owner"]).await;
    assert_eq!(owner["identity_id"], "usr-owner");
    assert_eq!(owner["administration"]["is_node_owner"], true);
    assert_eq!(owner["administration"]["is_node_admin"], true);
    assert_contains(&owner["capabilities"], "node_owner");
    assert_contains(&owner["capabilities"], "node_admin");
    assert_contains(&owner["administration"]["scopes"], "node_owner");

    let admin = get_capabilities(app.clone(), &tokens["usr-admin"]).await;
    assert_eq!(admin["identity_id"], "usr-admin");
    assert_eq!(admin["administration"]["is_node_owner"], false);
    assert_eq!(admin["administration"]["is_node_admin"], true);
    assert_contains(&admin["capabilities"], "node_admin");
    assert_contains(&admin["administration"]["scopes"], "node_operator");
    assert_not_contains(&admin["capabilities"], "node_owner");

    let member = get_capabilities(app, &tokens["usr-member"]).await;
    assert_eq!(member["identity_id"], "usr-member");
    assert_eq!(member["administration"]["is_node_owner"], false);
    assert_eq!(member["administration"]["is_node_admin"], false);
    assert_contains(&member["capabilities"], "node_connect");
    assert_contains(&member["capabilities"], "auth_session");
    assert_not_contains(&member["capabilities"], "node_admin");
    assert_eq!(
        member["administration"]["scopes"]
            .as_array()
            .expect("scopes array")
            .len(),
        0
    );
}

fn app_with_node_scope_sessions(
    owner_identity_ids: &[&str],
    admin_identity_ids: &[&str],
    session_identity_ids: &[&str],
) -> (axum::Router, HashMap<String, String>) {
    let state = test_state_with_public_identity_registration()
        .with_node_owner_identity_ids(
            owner_identity_ids
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        )
        .with_node_admin_identity_ids(
            admin_identity_ids
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        );
    let mut bearer_tokens = HashMap::new();

    {
        let mut sessions = state
            .sessions
            .write()
            .expect("acquire session write lock for node tests");

        for identity_id in session_identity_ids {
            let expires_at = Utc::now() + Duration::hours(1);
            let session_id = format!("sess-{identity_id}");

            sessions.insert(
                session_id.clone(),
                SessionRecord {
                    identity_id: (*identity_id).to_string(),
                    expires_at,
                },
            );

            bearer_tokens.insert(
                (*identity_id).to_string(),
                issue_session_token(
                    &session_id,
                    identity_id,
                    expires_at.timestamp(),
                    &state.active_signing_key_id,
                    state
                        .session_signing_keys
                        .get(&state.active_signing_key_id)
                        .expect("active signing key for node tests"),
                ),
            );
        }
    }

    (build_app(state), bearer_tokens)
}

async fn get_capabilities(app: axum::Router, bearer_token: &str) -> serde_json::Value {
    let request = Request::builder()
        .method("GET")
        .uri("/node/capabilities")
        .header("authorization", format!("Bearer {bearer_token}"))
        .body(Body::empty())
        .expect("build node capabilities request");

    let response = app
        .oneshot(request)
        .await
        .expect("node capabilities response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read node capabilities body");
    serde_json::from_slice(&body).expect("decode node capabilities body")
}

fn assert_contains(values: &serde_json::Value, expected: &str) {
    assert!(
        values
            .as_array()
            .expect("expected JSON array")
            .iter()
            .any(|value| value.as_str() == Some(expected)),
        "expected array to contain {expected}: {values:?}"
    );
}

fn assert_not_contains(values: &serde_json::Value, unexpected: &str) {
    assert!(
        values
            .as_array()
            .expect("expected JSON array")
            .iter()
            .all(|value| value.as_str() != Some(unexpected)),
        "expected array not to contain {unexpected}: {values:?}"
    );
}
