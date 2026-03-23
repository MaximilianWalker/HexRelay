use super::*;
use realtime_rs::{domain::presence::publish_online_if_needed, state::ConnectionSenderEntry};
use tokio::{net::TcpListener, sync::mpsc};

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
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        Vec::new(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        Some(redis_client.clone()),
        BTreeMap::from([(
            "v1".to_string(),
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
        .arg(format!("presence:v1:snapshot:{accepted}"))
        .arg(r#"{"status":"online"}"#)
        .query_async(&mut redis)
        .await
        .expect("set accepted presence snapshot");
    let _: () = redis::cmd("SET")
        .arg(format!("presence:v1:snapshot:{pending}"))
        .arg(r#"{"status":"online"}"#)
        .query_async(&mut redis)
        .await
        .expect("set pending presence snapshot");

    let token = issue_db_session_cookie(&pool, &state, &actor).await;
    let app = build_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/v1/contacts")
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
        .arg(format!("presence:v1:snapshot:{accepted}"))
        .arg(format!("presence:v1:snapshot:{pending}"))
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
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        Vec::new(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        Some(redis_client.clone()),
        BTreeMap::from([(
            "v1".to_string(),
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
    let presence_key = format!("presence:v1:snapshot:{accepted}");
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
            .uri("/v1/contacts")
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
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        Vec::new(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        Some(redis_client.clone()),
        BTreeMap::from([(
            "v1".to_string(),
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
        "hexrelay-dev-presence-token-change-me".to_string(),
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
            },
        )]),
    );

    let mut redis = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .expect("redis connection");
    let _: () = redis::cmd("DEL")
        .arg(format!("presence:v1:snapshot:{accepted}"))
        .arg(format!("presence:v1:watcher_stream_log:{actor}"))
        .arg(format!("presence:v1:watcher_stream_head:{actor}"))
        .arg(format!(
            "presence:v1:watcher_device_cursor:{actor}:device-primary"
        ))
        .arg(format!("presence:v1:count:{accepted}"))
        .arg(format!("presence:v1:seq:{accepted}"))
        .query_async(&mut redis)
        .await
        .expect("clear cross-service presence keys");

    publish_online_if_needed(&realtime_state, &accepted).await;

    let snapshot_raw: String = redis::cmd("GET")
        .arg(format!("presence:v1:snapshot:{accepted}"))
        .query_async(&mut redis)
        .await
        .expect("read realtime-written snapshot");
    let replay_entries: Vec<String> = redis::cmd("LRANGE")
        .arg(format!("presence:v1:watcher_stream_log:{actor}"))
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
        .get(format!("{api_base_url}/v1/contacts"))
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
        .arg(format!("presence:v1:snapshot:{accepted}"))
        .arg(format!("presence:v1:watcher_stream_log:{actor}"))
        .arg(format!("presence:v1:watcher_stream_head:{actor}"))
        .arg(format!(
            "presence:v1:watcher_device_cursor:{actor}:device-primary"
        ))
        .arg(format!("presence:v1:count:{accepted}"))
        .arg(format!("presence:v1:seq:{accepted}"))
        .query_async(&mut redis)
        .await
        .expect("clear cross-service presence keys");
}
