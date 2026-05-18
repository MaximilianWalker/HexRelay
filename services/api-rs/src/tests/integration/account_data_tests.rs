use super::*;

#[tokio::test]
async fn exports_account_data_scope_and_reports_dry_run_import_plan() {
    let actor = unique_identity("usr-account-export-a");
    let peer = unique_identity("usr-account-export-b");
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[actor.as_str(), peer.as_str()]).await
    else {
        return;
    };

    let request_id = format!("fr-account-export-{}", Uuid::new_v4().simple());
    sqlx::query(
        "
        INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status)
        VALUES ($1, $2, $3, 'accepted')
        ",
    )
    .bind(&request_id)
    .bind(&actor)
    .bind(&peer)
    .execute(&pool)
    .await
    .expect("insert accepted contact");

    sqlx::query(
        "
        INSERT INTO dm_profile_devices (identity_id, device_id, device_secret_hash, active, last_seen_epoch)
        VALUES ($1, 'desktop-primary', 'secret-hash-must-not-export', TRUE, 1779127803)
        ",
    )
    .bind(&actor)
    .execute(&pool)
    .await
    .expect("insert profile device");
    sqlx::query(
        "
        INSERT INTO dm_fanout_device_cursors (identity_id, device_id, cursor)
        VALUES ($1, 'desktop-primary', 7)
        ",
    )
    .bind(&actor)
    .execute(&pool)
    .await
    .expect("insert device cursor");

    let thread_id = format!("dm-account-export-{}", Uuid::new_v4().simple());
    let dm_message_actor = format!("dm-account-export-a-{}", Uuid::new_v4().simple());
    let dm_message_peer = format!("dm-account-export-b-{}", Uuid::new_v4().simple());
    let dm_participants = [(actor.as_str(), 1), (peer.as_str(), 0)];
    let dm_messages = [
        (
            dm_message_actor.as_str(),
            actor.as_str(),
            1,
            "enc:actor-owned",
            "2026-05-18T12:00:00Z",
            None,
        ),
        (
            dm_message_peer.as_str(),
            peer.as_str(),
            2,
            "enc:peer-visible",
            "2026-05-18T12:01:00Z",
            None,
        ),
    ];
    seed_dm_thread(
        &pool,
        &thread_id,
        "dm",
        "Account Export",
        &dm_participants,
        &dm_messages,
    )
    .await;

    let server_id = format!("srv-account-export-{}", Uuid::new_v4().simple());
    let channel_id = format!("chan-account-export-{}", Uuid::new_v4().simple());
    let server_message_id = format!("scm-account-export-{}", Uuid::new_v4().simple());
    let server_members = [actor.as_str(), peer.as_str()];
    let server_mentions = [peer.as_str()];
    let server_messages = [(
        server_message_id.as_str(),
        actor.as_str(),
        1,
        "owned server message",
        None,
        server_mentions.as_slice(),
        "2026-05-18T12:02:00Z",
        None,
        Some("2026-05-18T12:03:00Z"),
    )];
    seed_server_channel(
        &pool,
        &server_id,
        "Account Export Guild",
        &channel_id,
        "portable-data",
        &server_members,
        &server_messages,
    )
    .await;

    let export_request = Request::builder()
        .method("GET")
        .uri("/account/export")
        .header("cookie", format!("hexrelay_session={}", tokens[&actor]))
        .body(Body::empty())
        .expect("build account export request");

    let export_response = app
        .clone()
        .oneshot(export_request)
        .await
        .expect("account export response");
    assert_eq!(export_response.status(), StatusCode::OK);

    let export_body = to_bytes(export_response.into_body(), usize::MAX)
        .await
        .expect("read account export body");
    let export_text = std::str::from_utf8(&export_body).expect("export body is utf8");
    assert!(!export_text.contains("secret-hash-must-not-export"));
    assert!(!export_text.contains("device_secret_hash"));
    assert!(!export_text.contains("\"session_id\""));

    let export_payload: serde_json::Value =
        serde_json::from_slice(&export_body).expect("decode account export");
    assert_eq!(
        export_payload["kind"],
        serde_json::Value::String("hexrelay.account_data_export".to_string())
    );
    assert_eq!(export_payload["identity"]["identity_id"], actor);
    assert_eq!(
        export_payload["contacts"]
            .as_array()
            .expect("contacts")
            .len(),
        1
    );
    assert_eq!(
        export_payload["servers"].as_array().expect("servers").len(),
        1
    );
    assert_eq!(
        export_payload["dm_profile_devices"][0]["delivery_cursor"],
        serde_json::json!(7)
    );
    assert_eq!(
        export_payload["dm_threads"]
            .as_array()
            .expect("dm threads")
            .len(),
        1
    );
    assert_eq!(
        export_payload["dm_messages"]
            .as_array()
            .expect("dm messages")
            .len(),
        2
    );
    assert_eq!(
        export_payload["server_channel_messages"]
            .as_array()
            .expect("server messages")
            .len(),
        1
    );
    assert_eq!(
        export_payload["server_channel_messages"][0]["deleted_at"],
        serde_json::json!("2026-05-18T12:03:00Z")
    );

    let dry_run_body = serde_json::json!({
        "dry_run": true,
        "package": export_payload,
    });
    let dry_run_request = Request::builder()
        .method("POST")
        .uri("/account/import")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[&actor]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(dry_run_body.to_string()))
        .expect("build dry-run account import request");

    let dry_run_response = app
        .clone()
        .oneshot(dry_run_request)
        .await
        .expect("dry-run import response");
    assert_eq!(dry_run_response.status(), StatusCode::OK);

    let dry_run_body = to_bytes(dry_run_response.into_body(), usize::MAX)
        .await
        .expect("read dry-run import body");
    let dry_run_payload: serde_json::Value =
        serde_json::from_slice(&dry_run_body).expect("decode dry-run import");
    assert_eq!(dry_run_payload["status"], "dry_run");
    assert_eq!(dry_run_payload["mutating_import_available"], false);
    assert_eq!(dry_run_payload["planned_counts"]["dm_messages"], 2);
    assert_eq!(
        dry_run_payload["planned_counts"]["server_channel_messages"],
        1
    );

    let mutating_body = serde_json::json!({
        "dry_run": false,
        "package": dry_run_body_package(&dry_run_payload, &actor),
    });
    let mutating_request = Request::builder()
        .method("POST")
        .uri("/account/import")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[&actor]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(mutating_body.to_string()))
        .expect("build mutating account import request");

    let mutating_response = app
        .oneshot(mutating_request)
        .await
        .expect("mutating import response");
    assert_eq!(mutating_response.status(), StatusCode::BAD_REQUEST);

    let mutating_body = to_bytes(mutating_response.into_body(), usize::MAX)
        .await
        .expect("read mutating import body");
    let mutating_error: serde_json::Value =
        serde_json::from_slice(&mutating_body).expect("decode mutating import error");
    assert_eq!(
        mutating_error["code"],
        serde_json::json!("account_import_write_unavailable")
    );
}

fn dry_run_body_package(report: &serde_json::Value, identity_id: &str) -> serde_json::Value {
    serde_json::json!({
        "kind": "hexrelay.account_data_export",
        "generated_at": "2026-05-18T12:00:00Z",
        "identity": {
            "identity_id": identity_id,
            "public_key": "aa",
            "algorithm": "ed25519",
            "created_at": "2026-05-18T12:00:00Z"
        },
        "sessions": {
            "active_count": 1,
            "current_session_expires_at": "2026-05-18T13:00:00Z"
        },
        "contacts": [],
        "servers": [],
        "dm_profile_devices": [],
        "dm_threads": [],
        "dm_messages": [],
        "server_channel_messages": [],
        "retention": {
            "sessions": "session ids omitted",
            "dm_history": "ciphertext only",
            "dm_delivery_metadata": "not exported",
            "server_channel_messages": "authored messages only"
        },
        "limitations": report["warnings"].clone()
    })
}
