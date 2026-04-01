use super::*;
#[tokio::test]
async fn validates_and_revokes_db_backed_session() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let identity_id = unique_identity("db-user-verify");
    let app = register_identity_expect_success(app, &identity_id, &public_key).await;

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"{}"}}"#,
            identity_id
        )))
        .expect("build challenge request");

    let challenge_response = app
        .clone()
        .oneshot(challenge_request)
        .await
        .expect("challenge response");
    assert_eq!(challenge_response.status(), StatusCode::OK);

    let challenge_bytes = to_bytes(challenge_response.into_body(), usize::MAX)
        .await
        .expect("read challenge response body");
    let challenge: AuthChallengeResponse =
        serde_json::from_slice(&challenge_bytes).expect("decode challenge response");

    let signature = signing_key.sign(challenge.nonce.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"{}","challenge_id":"{}","signature":"{}"}}"#,
            identity_id, challenge.challenge_id, signature_hex
        )))
        .expect("build verify request");

    let verify_response = app
        .clone()
        .oneshot(verify_request)
        .await
        .expect("verify response");
    assert_eq!(verify_response.status(), StatusCode::OK);

    let session_cookie =
        extract_cookie_from_set_cookie_headers(&verify_response, "hexrelay_session")
            .expect("verify response includes session cookie");

    let verify_bytes = to_bytes(verify_response.into_body(), usize::MAX)
        .await
        .expect("read verify response body");
    let verify: AuthVerifyResponse =
        serde_json::from_slice(&verify_bytes).expect("decode verify response");

    let validate_request = Request::builder()
        .method("GET")
        .uri("/v1/auth/sessions/validate")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build validate request");

    let validate_response = app
        .clone()
        .oneshot(validate_request)
        .await
        .expect("validate response");
    assert_eq!(validate_response.status(), StatusCode::OK);

    let revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/sessions/revoke")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"session_id":"{}"}}"#,
            verify.session_id
        )))
        .expect("build revoke request");

    let revoke_response = app
        .clone()
        .oneshot(revoke_request)
        .await
        .expect("revoke response");
    assert_eq!(revoke_response.status(), StatusCode::NO_CONTENT);

    let validate_after_revoke = Request::builder()
        .method("GET")
        .uri("/v1/auth/sessions/validate")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build post-revoke validate request");

    let validate_after_revoke_response = app
        .oneshot(validate_after_revoke)
        .await
        .expect("post-revoke validate response");
    assert_eq!(
        validate_after_revoke_response.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn redeems_db_invite_after_app_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let identity_id = unique_identity("db-user-invite");
    let (session_cookie, app) = authenticate_identity(app, &identity_id).await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/invites")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"mode":"one_time"}"#))
        .expect("build create invite request");

    let create_response = app
        .oneshot(create_request)
        .await
        .expect("create invite response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create invite body");
    let created: InviteCreateResponse =
        serde_json::from_slice(&create_bytes).expect("decode invite create response");

    let Some(app_after_restart) = app_with_database().await else {
        return;
    };

    let redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            created.token, TEST_NODE_FINGERPRINT
        )))
        .expect("build redeem invite request");

    let redeem_response = app_after_restart
        .oneshot(redeem_request)
        .await
        .expect("redeem invite response");
    assert_eq!(redeem_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn verifies_db_challenge_after_app_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let identity_id = unique_identity("db-user-restart");
    let app = register_identity_expect_success(app, &identity_id, &public_key).await;

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"{}"}}"#,
            identity_id
        )))
        .expect("build challenge request");

    let challenge_response = app
        .oneshot(challenge_request)
        .await
        .expect("challenge response");
    assert_eq!(challenge_response.status(), StatusCode::OK);

    let challenge_bytes = to_bytes(challenge_response.into_body(), usize::MAX)
        .await
        .expect("read challenge response body");
    let challenge: AuthChallengeResponse =
        serde_json::from_slice(&challenge_bytes).expect("decode challenge response");

    let signature = signing_key.sign(challenge.nonce.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    let Some(app_after_restart) = app_with_database().await else {
        return;
    };

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"{}","challenge_id":"{}","signature":"{}"}}"#,
            identity_id, challenge.challenge_id, signature_hex
        )))
        .expect("build verify request");

    let verify_response = app_after_restart
        .oneshot(verify_request)
        .await
        .expect("verify response");
    assert_eq!(verify_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rejects_replayed_pairing_envelope_nonce_after_app_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let inviter_identity = unique_identity("db-dm-pair-inviter");
    let first_importer_identity = unique_identity("db-dm-pair-importer-a");
    let second_importer_identity = unique_identity("db-dm-pair-importer-b");

    let (inviter_cookie, app) = authenticate_identity(app, &inviter_identity).await;
    let (first_importer_cookie, app) = authenticate_identity(app, &first_importer_identity).await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={inviter_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
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

    let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read pairing envelope create body");
    let create_payload: serde_json::Value =
        serde_json::from_slice(&create_bytes).expect("decode pairing envelope create body");
    let envelope = create_payload["envelope"]
        .as_str()
        .expect("pairing envelope payload present")
        .to_string();

    let first_import_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope/import")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={first_importer_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(r#"{{"envelope":"{envelope}"}}"#)))
        .expect("build first pairing envelope import request");
    let first_import_response = app
        .oneshot(first_import_request)
        .await
        .expect("first pairing envelope import response");
    assert_eq!(first_import_response.status(), StatusCode::OK);

    let Some(app_after_restart) = app_with_database().await else {
        return;
    };
    let (second_importer_cookie, app_after_restart) =
        authenticate_identity(app_after_restart, &second_importer_identity).await;

    let second_import_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/pairing-envelope/import")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={second_importer_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(r#"{{"envelope":"{envelope}"}}"#)))
        .expect("build second pairing envelope import request");
    let second_import_response = app_after_restart
        .oneshot(second_import_request)
        .await
        .expect("second pairing envelope import response");
    assert_eq!(second_import_response.status(), StatusCode::BAD_REQUEST);

    let second_import_body = to_bytes(second_import_response.into_body(), usize::MAX)
        .await
        .expect("read second pairing envelope import body");
    let second_import_payload: serde_json::Value = serde_json::from_slice(&second_import_body)
        .expect("decode second pairing envelope import body");
    assert_eq!(second_import_payload["code"], "pairing_replayed");
}

#[tokio::test]
async fn persists_friend_flow_and_lists_contacts_from_db() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let requester_identity = unique_identity("db-user-friends-a");
    let target_identity = unique_identity("db-user-friends-b");
    let (requester_cookie, app) = authenticate_identity(app, &requester_identity).await;
    let (target_cookie, app) = authenticate_identity(app, &target_identity).await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={requester_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"requester_identity_id":"{}","target_identity_id":"{}"}}"#,
            requester_identity, target_identity
        )))
        .expect("build create friend request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create friend request response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create friend request body");
    let created: FriendRequestRecord =
        serde_json::from_slice(&create_body).expect("decode create friend request body");

    let accept_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header(
            "cookie",
            format!("hexrelay_session={target_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::empty())
        .expect("build accept friend request");

    let accept_response = app
        .clone()
        .oneshot(accept_request)
        .await
        .expect("accept friend request response");
    assert_eq!(accept_response.status(), StatusCode::OK);

    let contacts_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/contacts?search={}", target_identity))
        .header("cookie", format!("hexrelay_session={requester_cookie}"))
        .body(Body::empty())
        .expect("build contacts request");

    let contacts_response = app
        .oneshot(contacts_request)
        .await
        .expect("contacts response");
    assert_eq!(contacts_response.status(), StatusCode::OK);

    let contacts_body = to_bytes(contacts_response.into_body(), usize::MAX)
        .await
        .expect("read contacts response body");
    let payload: ContactListResponse =
        serde_json::from_slice(&contacts_body).expect("decode contacts response body");

    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0]["id"], target_identity);
    assert_eq!(payload.items[0]["inbound_request"], false);
    assert_eq!(payload.items[0]["pending_request"], false);
}

#[tokio::test]
async fn preflight_friends_only_uses_db_friendship_state() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let requester_identity = unique_identity("db-dm-policy-a");
    let target_identity = unique_identity("db-dm-policy-b");
    let (requester_cookie, app) = authenticate_identity(app, &requester_identity).await;
    let (target_cookie, app) = authenticate_identity(app, &target_identity).await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={requester_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"requester_identity_id":"{}","target_identity_id":"{}"}}"#,
            requester_identity, target_identity
        )))
        .expect("build create friend request");
    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create friend request response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create friend request body");
    let created: FriendRequestRecord =
        serde_json::from_slice(&create_body).expect("decode create friend request body");

    let accept_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header(
            "cookie",
            format!("hexrelay_session={target_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::empty())
        .expect("build accept friend request");
    let accept_response = app
        .clone()
        .oneshot(accept_request)
        .await
        .expect("accept friend request response");
    assert_eq!(accept_response.status(), StatusCode::OK);

    let preflight_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={target_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"pairing_envelope_present":true,"peer_identity_id":"{}","local_bind_allowed":true,"peer_reachable_hint":true}}"#,
            requester_identity
        )))
        .expect("build dm preflight request");
    let preflight_response = app
        .oneshot(preflight_request)
        .await
        .expect("dm preflight response");
    assert_eq!(preflight_response.status(), StatusCode::OK);

    let preflight_body = to_bytes(preflight_response.into_body(), usize::MAX)
        .await
        .expect("read dm preflight body");
    let preflight_payload: serde_json::Value =
        serde_json::from_slice(&preflight_body).expect("decode dm preflight payload");
    assert_eq!(preflight_payload["status"], "ready");
    assert_eq!(preflight_payload["reason_code"], "preflight_ok");
}

#[tokio::test]
async fn redeems_contact_invite_in_db_and_is_idempotent() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let inviter_identity = unique_identity("db-user-contact-invite-a");
    let redeemer_identity = unique_identity("db-user-contact-invite-b");
    let (inviter_cookie, app) = authenticate_identity(app, &inviter_identity).await;
    let (redeemer_cookie, app) = authenticate_identity(app, &redeemer_identity).await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/contact-invites")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={inviter_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"mode":"multi_use","max_uses":3}"#))
        .expect("build create contact invite request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create contact invite response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create contact invite body");
    let created: InviteCreateResponse =
        serde_json::from_slice(&create_body).expect("decode create contact invite body");

    let first_redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/contact-invites/redeem")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={redeemer_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(r#"{{"token":"{}"}}"#, created.token)))
        .expect("build first contact invite redeem request");

    let first_redeem_response = app
        .clone()
        .oneshot(first_redeem_request)
        .await
        .expect("first contact invite redeem response");
    assert_eq!(first_redeem_response.status(), StatusCode::OK);

    let first_redeem_body = to_bytes(first_redeem_response.into_body(), usize::MAX)
        .await
        .expect("read first contact invite redeem body");
    let first_friend_request: FriendRequestRecord =
        serde_json::from_slice(&first_redeem_body).expect("decode first contact friend request");
    assert_eq!(
        first_friend_request.requester_identity_id,
        redeemer_identity
    );
    assert_eq!(first_friend_request.target_identity_id, inviter_identity);
    assert_eq!(first_friend_request.status, "pending");

    let second_redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/contact-invites/redeem")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={redeemer_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(r#"{{"token":"{}"}}"#, created.token)))
        .expect("build second contact invite redeem request");

    let second_redeem_response = app
        .oneshot(second_redeem_request)
        .await
        .expect("second contact invite redeem response");
    assert_eq!(second_redeem_response.status(), StatusCode::OK);

    let second_redeem_body = to_bytes(second_redeem_response.into_body(), usize::MAX)
        .await
        .expect("read second contact invite redeem body");
    let second_friend_request: FriendRequestRecord =
        serde_json::from_slice(&second_redeem_body).expect("decode second contact friend request");

    assert_eq!(
        first_friend_request.request_id,
        second_friend_request.request_id
    );
}

#[tokio::test]
async fn rejects_redeeming_own_contact_invite_in_db() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let inviter_identity = unique_identity("db-user-contact-invite-self");
    let (inviter_cookie, app) = authenticate_identity(app, &inviter_identity).await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/contact-invites")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={inviter_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"mode":"multi_use","max_uses":3}"#))
        .expect("build create contact invite request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create contact invite response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create contact invite body");
    let created: InviteCreateResponse =
        serde_json::from_slice(&create_body).expect("decode create contact invite body");

    let redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/contact-invites/redeem")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={inviter_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(r#"{{"token":"{}"}}"#, created.token)))
        .expect("build contact invite redeem request");

    let redeem_response = app
        .oneshot(redeem_request)
        .await
        .expect("contact invite redeem response");

    assert_eq!(redeem_response.status(), StatusCode::CONFLICT);
}
