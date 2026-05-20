use super::*;
use realtime_rs::{domain::presence::publish_online_if_needed, state::ConnectionSenderEntry};
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::test]
async fn lists_servers_with_filters_from_persisted_memberships() {
    let identity_id = unique_identity("usr-server-filters");
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&identity_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Atlas Core", &identity_id, true, false, 2).await;

    let request = Request::builder()
        .method("GET")
        .uri("/servers?pinned_only=true&unread_only=true")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[&identity_id]),
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

    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0]["id"], TEST_SERVER_ID);
    assert!(payload.items.iter().all(|item| item["pinned"] == true));
    assert!(payload
        .items
        .iter()
        .all(|item| item["unread"].as_u64().unwrap_or_default() > 0));
}

#[tokio::test]
async fn lists_servers_for_authenticated_identity_only() {
    let nora_id = unique_identity("usr-list-nora");
    let alex_id = unique_identity("usr-list-alex");
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&nora_id, &alex_id]).await
    else {
        return;
    };

    seed_server_membership(&pool, "Atlas Core", &nora_id, true, false, 2).await;

    let nora_request = Request::builder()
        .method("GET")
        .uri("/servers")
        .header("cookie", format!("hexrelay_session={}", tokens[&nora_id]))
        .body(Body::empty())
        .expect("build nora servers list request");
    let alex_request = Request::builder()
        .method("GET")
        .uri("/servers")
        .header("cookie", format!("hexrelay_session={}", tokens[&alex_id]))
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
        .any(|item| item["id"] == TEST_SERVER_ID));
    assert!(nora_payload
        .items
        .iter()
        .all(|item| item["id"] == TEST_SERVER_ID));

    assert!(alex_payload.items.is_empty());
}

#[tokio::test]
async fn updates_server_preferences_and_leave_removes_membership() {
    let member_id = unique_identity("usr-server-actions");
    let server_id = TEST_SERVER_ID.to_string();
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Action Server", &member_id, false, false, 0).await;

    let update_request = Request::builder()
        .method("PATCH")
        .uri(format!("/servers/{server_id}/preferences"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::from(r#"{"pinned":true,"muted":true}"#))
        .expect("build server preference request");
    let update_response = app
        .clone()
        .oneshot(update_request)
        .await
        .expect("server preference response");
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_body = to_bytes(update_response.into_body(), usize::MAX)
        .await
        .expect("read server preference body");
    let update_payload: serde_json::Value =
        serde_json::from_slice(&update_body).expect("decode server preference body");
    assert_eq!(update_payload["item"]["pinned"], true);
    assert_eq!(update_payload["item"]["muted"], true);

    let leave_request = Request::builder()
        .method("POST")
        .uri(format!("/servers/{server_id}/leave"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::from(r#"{"delete_local_data":true}"#))
        .expect("build server leave request");
    let leave_response = app
        .clone()
        .oneshot(leave_request)
        .await
        .expect("server leave response");
    assert_eq!(leave_response.status(), StatusCode::OK);
    let leave_body = to_bytes(leave_response.into_body(), usize::MAX)
        .await
        .expect("read server leave body");
    let leave_payload: serde_json::Value =
        serde_json::from_slice(&leave_body).expect("decode server leave body");
    assert_eq!(leave_payload["left"], true);
    assert_eq!(leave_payload["deleted_local_data"], true);

    let list_request = Request::builder()
        .method("GET")
        .uri("/servers")
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build server list request");
    let list_response = app
        .oneshot(list_request)
        .await
        .expect("server list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read server list body");
    let list_payload: ServerListResponse =
        serde_json::from_slice(&list_body).expect("decode server list");
    assert!(list_payload.items.is_empty());
}

#[tokio::test]
async fn create_server_claims_owner_and_seeds_text_and_voice_channels() {
    let owner_id = unique_identity("usr-server-owner-create");
    let server_id = TEST_SERVER_ID.to_string();
    let Some((app, tokens, _pool)) = app_with_database_and_sessions(&[&owner_id]).await else {
        return;
    };

    let create_request = Request::builder()
        .method("POST")
        .uri("/servers")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens[&owner_id]))
        .body(Body::from(
            r#"{"name":"Atlas Local","description":"Local server"}"#,
        ))
        .expect("build server create request");
    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("server create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read server create body");
    let create_payload: serde_json::Value =
        serde_json::from_slice(&create_body).expect("decode server create body");
    assert_eq!(create_payload["item"]["name"], "Atlas Local");
    assert_eq!(create_payload["item"]["pinned"], true);
    assert_eq!(
        create_payload["owner_identity_id"].as_str(),
        Some(owner_id.as_str())
    );
    assert!(create_payload["bootstrap_credential"]
        .as_str()
        .is_some_and(|value| value.starts_with("srv-bootstrap-")));

    let capabilities_request = Request::builder()
        .method("GET")
        .uri("/server/capabilities")
        .header("authorization", format!("Bearer {}", tokens[&owner_id]))
        .body(Body::empty())
        .expect("build server capabilities request");
    let capabilities_response = app
        .clone()
        .oneshot(capabilities_request)
        .await
        .expect("server capabilities response");
    assert_eq!(capabilities_response.status(), StatusCode::OK);
    let capabilities_body = to_bytes(capabilities_response.into_body(), usize::MAX)
        .await
        .expect("read capabilities body");
    let capabilities_payload: serde_json::Value =
        serde_json::from_slice(&capabilities_body).expect("decode capabilities body");
    assert_eq!(
        capabilities_payload["administration"]["is_server_owner"],
        true
    );

    let channels_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}/channels"))
        .header("authorization", format!("Bearer {}", tokens[&owner_id]))
        .body(Body::empty())
        .expect("build server channels request");
    let channels_response = app
        .oneshot(channels_request)
        .await
        .expect("server channels response");
    assert_eq!(channels_response.status(), StatusCode::OK);
    let channels_body = to_bytes(channels_response.into_body(), usize::MAX)
        .await
        .expect("read channels body");
    let channels_payload: serde_json::Value =
        serde_json::from_slice(&channels_body).expect("decode channels body");
    let kinds = channels_payload["items"]
        .as_array()
        .expect("items array")
        .iter()
        .filter_map(|item| item["kind"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(kinds.contains("text"));
    assert!(kinds.contains("voice"));
}

#[tokio::test]
async fn join_server_accepts_invite_link_and_adds_membership() {
    let owner_id = unique_identity("usr-server-join-owner");
    let joiner_id = unique_identity("usr-server-joiner");
    let Some((app, tokens, _pool)) = app_with_database_and_sessions(&[&owner_id, &joiner_id]).await
    else {
        return;
    };

    let invite_request = Request::builder()
        .method("POST")
        .uri("/invites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens[&owner_id]))
        .body(Body::from(r#"{"mode":"multi_use","max_uses":4}"#))
        .expect("build invite request");
    let invite_response = app
        .clone()
        .oneshot(invite_request)
        .await
        .expect("invite response");
    assert_eq!(invite_response.status(), StatusCode::CREATED);
    let invite_body = to_bytes(invite_response.into_body(), usize::MAX)
        .await
        .expect("read invite body");
    let invite_payload: serde_json::Value =
        serde_json::from_slice(&invite_body).expect("decode invite body");
    let token = invite_payload["token"].as_str().expect("invite token");

    let join_body = serde_json::json!({
        "invite_link": format!("hexrelay://join?server_id={TEST_SERVER_ID}&token={token}")
    });
    let join_request = Request::builder()
        .method("POST")
        .uri("/servers/join")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens[&joiner_id]))
        .body(Body::from(join_body.to_string()))
        .expect("build join request");
    let join_response = app.oneshot(join_request).await.expect("join response");
    assert_eq!(join_response.status(), StatusCode::OK);
    let join_body = to_bytes(join_response.into_body(), usize::MAX)
        .await
        .expect("read join body");
    let join_payload: serde_json::Value =
        serde_json::from_slice(&join_body).expect("decode join body");
    assert_eq!(join_payload["joined"], true);
    assert_eq!(join_payload["item"]["id"], TEST_SERVER_ID);
}

#[tokio::test]
async fn lists_contacts_with_search_filter() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);
    let request = Request::builder()
        .method("GET")
        .uri("/contacts?search=nora")
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

#[tokio::test]
async fn updates_contact_preferences_and_block_remove_hides_contact() {
    let actor = unique_identity("usr-contact-action-actor");
    let peer = unique_identity("usr-contact-action-peer");
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&actor, &peer]).await else {
        return;
    };

    sqlx::query(
        "INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status) VALUES ($1, $2, $3, 'accepted')",
    )
    .bind(format!("fr-{}", Uuid::new_v4().simple()))
    .bind(&actor)
    .bind(&peer)
    .execute(&pool)
    .await
    .expect("insert accepted friend request");

    let preference_request = Request::builder()
        .method("PATCH")
        .uri(format!("/contacts/{peer}/preferences"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens[&actor]))
        .body(Body::from(r#"{"pinned":true,"muted":true}"#))
        .expect("build contact preference request");
    let preference_response = app
        .clone()
        .oneshot(preference_request)
        .await
        .expect("contact preference response");
    assert_eq!(preference_response.status(), StatusCode::OK);
    let preference_body = to_bytes(preference_response.into_body(), usize::MAX)
        .await
        .expect("read contact preference body");
    let preference_payload: serde_json::Value =
        serde_json::from_slice(&preference_body).expect("decode contact preference body");
    assert_eq!(preference_payload["pinned"], true);
    assert_eq!(preference_payload["muted"], true);

    let pinned_list_request = Request::builder()
        .method("GET")
        .uri("/contacts?pinned_only=true&muted_only=true")
        .header("authorization", format!("Bearer {}", tokens[&actor]))
        .body(Body::empty())
        .expect("build pinned contacts request");
    let pinned_list_response = app
        .clone()
        .oneshot(pinned_list_request)
        .await
        .expect("pinned contacts response");
    assert_eq!(pinned_list_response.status(), StatusCode::OK);
    let pinned_list_body = to_bytes(pinned_list_response.into_body(), usize::MAX)
        .await
        .expect("read pinned contacts body");
    let pinned_payload: ContactListResponse =
        serde_json::from_slice(&pinned_list_body).expect("decode pinned contacts");
    assert_eq!(pinned_payload.items.len(), 1);
    assert_eq!(pinned_payload.items[0]["id"].as_str(), Some(peer.as_str()));

    let block_request = Request::builder()
        .method("POST")
        .uri(format!("/contacts/{peer}/block-remove"))
        .header("authorization", format!("Bearer {}", tokens[&actor]))
        .body(Body::empty())
        .expect("build block-remove request");
    let block_response = app
        .clone()
        .oneshot(block_request)
        .await
        .expect("block-remove response");
    assert_eq!(block_response.status(), StatusCode::OK);
    let block_body = to_bytes(block_response.into_body(), usize::MAX)
        .await
        .expect("read block-remove body");
    let block_payload: serde_json::Value =
        serde_json::from_slice(&block_body).expect("decode block-remove body");
    assert_eq!(
        block_payload["blocked_identity_id"].as_str(),
        Some(peer.as_str())
    );
    assert_eq!(block_payload["relationship_removed"], true);

    let list_request = Request::builder()
        .method("GET")
        .uri("/contacts")
        .header("authorization", format!("Bearer {}", tokens[&actor]))
        .body(Body::empty())
        .expect("build contacts list request");
    let list_response = app
        .oneshot(list_request)
        .await
        .expect("contacts list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read contacts list body");
    let list_payload: ContactListResponse =
        serde_json::from_slice(&list_body).expect("decode contacts list");
    assert!(list_payload.items.is_empty());
}

#[tokio::test]
async fn lists_contacts_with_redis_presence_snapshots_for_accepted_contacts_only() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let Some(redis_client) = prepared_presence_redis_client().await else {
        return;
    };

    let actor = unique_identity("usr-contacts-actor");
    let accepted = unique_identity("usr-contacts-accepted");
    let pending = unique_identity("usr-contacts-pending");

    let state = AppState::new(
        TEST_SERVER_ID.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "primary".to_string(),
        Vec::new(),
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        Some(redis_client.clone()),
        "http://127.0.0.1:8081".to_string(),
        BTreeMap::from([(
            "primary".to_string(),
            "hexrelay-dev-signing-key-change-me".to_string(),
        )]),
        None,
        false,
        "Lax".to_string(),
        ApiRateLimitConfig {
            auth_challenge_per_window: 30,
            auth_verify_per_window: 30,
            discovery_query_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            dm_dispatch_per_window: 120,
            dm_catch_up_per_window: 120,
            dm_ack_per_window: 600,
            dm_internal_forward_per_window: 240,
            window_seconds: 60,
        },
        false,
    )
    .with_db_pool(pool.clone());

    ensure_db_identity_key(&pool, &actor).await;
    ensure_db_identity_key(&pool, &accepted).await;
    ensure_db_identity_key(&pool, &pending).await;

    sqlx::query(
        "INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status) VALUES ($1, $2, $3, 'accepted')",
    )
    .bind(format!("fr-{}", Uuid::new_v4().simple()))
    .bind(&actor)
    .bind(&accepted)
    .execute(&pool)
    .await
    .expect("insert accepted friend request");
    sqlx::query(
        "INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status) VALUES ($1, $2, $3, 'pending')",
    )
    .bind(format!("fr-{}", Uuid::new_v4().simple()))
    .bind(&actor)
    .bind(&pending)
    .execute(&pool)
    .await
    .expect("insert pending friend request");

    let mut redis = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .expect("redis connection");
    let _: () = redis::cmd("SET")
        .arg(format!("presence:snapshot:{accepted}"))
        .arg(r#"{"status":"online"}"#)
        .query_async(&mut redis)
        .await
        .expect("set accepted presence snapshot");
    let _: () = redis::cmd("SET")
        .arg(format!("presence:snapshot:{pending}"))
        .arg(r#"{"status":"online"}"#)
        .query_async(&mut redis)
        .await
        .expect("set pending presence snapshot");

    let token = issue_db_session_cookie(&pool, &state, &actor).await;
    let app = build_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/contacts")
        .header("cookie", format!("hexrelay_session={token}"))
        .body(Body::empty())
        .expect("build contacts request");

    let response = app.oneshot(request).await.expect("contacts response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read contacts response body");
    let payload: ContactListResponse =
        serde_json::from_slice(&body).expect("decode contacts payload");

    let accepted_item = payload
        .items
        .iter()
        .find(|item| item["id"] == accepted)
        .expect("accepted contact present");
    let pending_item = payload
        .items
        .iter()
        .find(|item| item["id"] == pending)
        .expect("pending contact present");

    assert_eq!(accepted_item["status"], "online");
    assert_eq!(pending_item["status"], "offline");
    assert_eq!(pending_item["pending_request"], true);

    let _: () = redis::cmd("DEL")
        .arg(format!("presence:snapshot:{accepted}"))
        .arg(format!("presence:snapshot:{pending}"))
        .query_async(&mut redis)
        .await
        .expect("clear presence snapshots");
}

#[tokio::test]
async fn lists_contacts_returns_latest_converged_presence_snapshot_after_reconnect_sequence() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let Some(redis_client) = prepared_presence_redis_client().await else {
        return;
    };

    let actor = unique_identity("usr-contacts-actor-converged");
    let accepted = unique_identity("usr-contacts-accepted-converged");

    let state = AppState::new(
        TEST_SERVER_ID.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "primary".to_string(),
        Vec::new(),
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        Some(redis_client.clone()),
        "http://127.0.0.1:8081".to_string(),
        BTreeMap::from([(
            "primary".to_string(),
            "hexrelay-dev-signing-key-change-me".to_string(),
        )]),
        None,
        false,
        "Lax".to_string(),
        ApiRateLimitConfig {
            auth_challenge_per_window: 30,
            auth_verify_per_window: 30,
            discovery_query_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            dm_dispatch_per_window: 120,
            dm_catch_up_per_window: 120,
            dm_ack_per_window: 600,
            dm_internal_forward_per_window: 240,
            window_seconds: 60,
        },
        false,
    )
    .with_db_pool(pool.clone());

    ensure_db_identity_key(&pool, &actor).await;
    ensure_db_identity_key(&pool, &accepted).await;

    sqlx::query(
        "INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status) VALUES ($1, $2, $3, 'accepted')",
    )
    .bind(format!("fr-{}", Uuid::new_v4().simple()))
    .bind(&actor)
    .bind(&accepted)
    .execute(&pool)
    .await
    .expect("insert accepted friend request");

    let mut redis = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .expect("redis connection");
    let presence_key = format!("presence:snapshot:{accepted}");
    for (status, seq) in [("online", 1_u64), ("offline", 2_u64), ("online", 3_u64)] {
        let payload = serde_json::json!({
            "status": status,
            "updated_at": format!("2026-03-23T00:00:0{seq}Z"),
            "presence_seq": seq,
        })
        .to_string();
        let _: () = redis::cmd("SET")
            .arg(&presence_key)
            .arg(payload)
            .query_async(&mut redis)
            .await
            .expect("set converged presence snapshot");
        let token = issue_db_session_cookie(&pool, &state, &actor).await;
        let app = build_app(state.clone());

        let request = Request::builder()
            .method("GET")
            .uri("/contacts")
            .header("cookie", format!("hexrelay_session={token}"))
            .body(Body::empty())
            .expect("build contacts request");

        let response = app.oneshot(request).await.expect("contacts response");
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read contacts response body");
        let payload: ContactListResponse =
            serde_json::from_slice(&body).expect("decode contacts payload");

        let accepted_item = payload
            .items
            .iter()
            .find(|item| item["id"] == accepted)
            .expect("accepted contact present");

        assert_eq!(accepted_item["status"], status);
    }

    let _: () = redis::cmd("DEL")
        .arg(&presence_key)
        .query_async(&mut redis)
        .await
        .expect("clear converged presence snapshot");
}

#[tokio::test]
async fn lists_contacts_reads_snapshot_written_by_realtime_presence_publish_path() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let Some(redis_client) = prepared_presence_redis_client().await else {
        return;
    };

    let actor = unique_identity("usr-contacts-actor-cross-service");
    let accepted = unique_identity("usr-contacts-accepted-cross-service");

    let api_state = AppState::new(
        TEST_SERVER_ID.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "primary".to_string(),
        Vec::new(),
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        Some(redis_client.clone()),
        "http://127.0.0.1:8081".to_string(),
        BTreeMap::from([(
            "primary".to_string(),
            "hexrelay-dev-signing-key-change-me".to_string(),
        )]),
        None,
        false,
        "Lax".to_string(),
        ApiRateLimitConfig {
            auth_challenge_per_window: 30,
            auth_verify_per_window: 30,
            discovery_query_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            dm_dispatch_per_window: 120,
            dm_catch_up_per_window: 120,
            dm_ack_per_window: 600,
            dm_internal_forward_per_window: 240,
            window_seconds: 60,
        },
        false,
    )
    .with_db_pool(pool.clone());

    ensure_db_identity_key(&pool, &actor).await;
    ensure_db_identity_key(&pool, &accepted).await;
    sqlx::query(
        "INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status) VALUES ($1, $2, $3, 'accepted')",
    )
    .bind(format!("fr-{}", Uuid::new_v4().simple()))
    .bind(&actor)
    .bind(&accepted)
    .execute(&pool)
    .await
    .expect("insert accepted friend request");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind api test server");
    let api_addr = listener.local_addr().expect("api test server addr");
    let api_app = build_app(api_state.clone());
    tokio::spawn(async move {
        axum::serve(listener, api_app)
            .await
            .expect("serve api test app");
    });
    let api_base_url = format!("http://{}", api_addr);

    let realtime_state = realtime_rs::state::AppState::new(
        api_base_url.clone(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        Some(redis_client.clone()),
        false,
        60,
        60,
        16_384,
        120,
        60,
        3,
        0,
        10_000,
    )
    .expect("build realtime state");
    let (sender, _receiver) = mpsc::channel::<String>(4);
    realtime_state.connection_senders.lock().await.insert(
        actor.clone(),
        HashMap::from([(
            "conn-primary".to_string(),
            ConnectionSenderEntry {
                sender,
                device_id: Some("device-primary".to_string()),
                dm_device_verified: false,
            },
        )]),
    );

    let mut redis = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .expect("redis connection");
    let _: () = redis::cmd("DEL")
        .arg(format!("presence:snapshot:{accepted}"))
        .arg(format!("presence:watcher_stream_log:{actor}"))
        .arg(format!("presence:watcher_stream_head:{actor}"))
        .arg(format!(
            "presence:watcher_device_cursor:{actor}:device-primary"
        ))
        .arg(format!("presence:count:{accepted}"))
        .arg(format!("presence:seq:{accepted}"))
        .query_async(&mut redis)
        .await
        .expect("clear cross-service presence keys");

    publish_online_if_needed(&realtime_state, &accepted).await;

    let snapshot_raw: String = redis::cmd("GET")
        .arg(format!("presence:snapshot:{accepted}"))
        .query_async(&mut redis)
        .await
        .expect("read realtime-written snapshot");
    let replay_entries: Vec<String> = redis::cmd("LRANGE")
        .arg(format!("presence:watcher_stream_log:{actor}"))
        .arg(0)
        .arg(-1)
        .query_async(&mut redis)
        .await
        .expect("read watcher replay log");
    assert_eq!(replay_entries.len(), 1);

    let snapshot_json: serde_json::Value =
        serde_json::from_str(&snapshot_raw).expect("decode realtime snapshot json");
    let replay_entry_json: serde_json::Value =
        serde_json::from_str(&replay_entries[0]).expect("decode replay entry json");
    let replay_payload_json: serde_json::Value = serde_json::from_str(
        replay_entry_json["payload"]
            .as_str()
            .expect("payload string"),
    )
    .expect("decode replay payload json");

    assert_eq!(
        snapshot_json["status"],
        replay_payload_json["data"]["status"]
    );
    assert_eq!(
        snapshot_json["presence_seq"],
        replay_payload_json["data"]["presence_seq"]
    );
    assert_eq!(
        snapshot_json["updated_at"],
        replay_payload_json["data"]["updated_at"]
    );

    let token = issue_db_session_cookie(&pool, &api_state, &actor).await;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build api test client");
    let response = client
        .get(format!("{api_base_url}/contacts"))
        .header(reqwest::header::COOKIE, format!("hexrelay_session={token}"))
        .send()
        .await
        .expect("request contacts from api server");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let payload: ContactListResponse = response
        .json()
        .await
        .expect("decode contacts payload from api server");
    let accepted_item = payload
        .items
        .iter()
        .find(|item| item["id"] == accepted)
        .expect("accepted contact present");
    assert_eq!(accepted_item["status"], "online");

    let _: () = redis::cmd("DEL")
        .arg(format!("presence:snapshot:{accepted}"))
        .arg(format!("presence:watcher_stream_log:{actor}"))
        .arg(format!("presence:watcher_stream_head:{actor}"))
        .arg(format!(
            "presence:watcher_device_cursor:{actor}:device-primary"
        ))
        .arg(format!("presence:count:{accepted}"))
        .arg(format!("presence:seq:{accepted}"))
        .query_async(&mut redis)
        .await
        .expect("clear cross-service presence keys");
}
