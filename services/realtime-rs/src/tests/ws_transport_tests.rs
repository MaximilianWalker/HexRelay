use axum::{
    extract::Path,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, env, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message as WsMessage},
};

use crate::app::{build_app, AppState};
use crate::domain::channels::{publish_channel_message_created, PublishChannelMessageCreatedInput};
use crate::domain::{channels::spawn_channel_subscriber, presence::spawn_presence_subscriber};

use crate::transport::ws::handlers::gateway::{is_session_valid, route_inbound_event};

const TEST_ALLOWED_ORIGIN: &str = "http://localhost:3002";

fn test_allowed_origins() -> Vec<String> {
    vec![TEST_ALLOWED_ORIGIN.to_string()]
}

fn set_allowed_origin(request: &mut tokio_tungstenite::tungstenite::http::Request<()>) {
    request
        .headers_mut()
        .insert("origin", HeaderValue::from_static(TEST_ALLOWED_ORIGIN));
}

fn auth_cache_key(authorization_value: &str) -> String {
    let digest = Sha256::digest(authorization_value.as_bytes());
    let mut first_eight = [0_u8; 8];
    first_eight.copy_from_slice(&digest[..8]);
    format!("auth:{:016x}", u64::from_be_bytes(first_eight))
}

#[derive(Serialize)]
struct ValidatePayload {
    session_id: String,
    identity_id: String,
    expires_at: String,
}

#[derive(Clone, Copy)]
enum ValidateMode {
    Authorized,
    Denied,
    Unavailable,
}

#[derive(Clone)]
struct PresenceApiStubState {
    sessions: Arc<RwLock<HashMap<String, String>>>,
    watchers: Arc<RwLock<HashMap<String, Vec<String>>>>,
    internal_token: String,
}

async fn start_validate_server(mode: ValidateMode) -> String {
    async fn validate_endpoint(
        State(mode): State<ValidateMode>,
        headers: HeaderMap,
    ) -> (StatusCode, Json<ValidatePayload>) {
        let authorized = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(|value| value == "Bearer test-token")
            .unwrap_or(false)
            || headers
                .get("cookie")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.contains("hexrelay_session=test-token"))
                .unwrap_or(false);

        if !authorized {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ValidatePayload {
                    session_id: String::new(),
                    identity_id: String::new(),
                    expires_at: String::new(),
                }),
            );
        }

        let payload = ValidatePayload {
            session_id: "sess-1".to_string(),
            identity_id: "usr-1".to_string(),
            expires_at: "2030-01-01T00:00:00Z".to_string(),
        };

        match mode {
            ValidateMode::Authorized => (StatusCode::OK, Json(payload)),
            ValidateMode::Denied => (StatusCode::UNAUTHORIZED, Json(payload)),
            ValidateMode::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, Json(payload)),
        }
    }

    let app = Router::new()
        .route("/v1/auth/sessions/validate", get(validate_endpoint))
        .with_state(mode);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let address = listener.local_addr().expect("read listener address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test API");
    });

    format!("http://{}", address)
}

async fn start_presence_api_stub(
    sessions: HashMap<String, String>,
    watchers: HashMap<String, Vec<String>>,
    internal_token: &str,
) -> String {
    async fn validate_endpoint(
        State(state): State<PresenceApiStubState>,
        headers: HeaderMap,
    ) -> (StatusCode, Json<ValidatePayload>) {
        let Some(authorization) = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
        else {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ValidatePayload {
                    session_id: String::new(),
                    identity_id: String::new(),
                    expires_at: String::new(),
                }),
            );
        };

        let identity_id = state.sessions.read().await.get(authorization).cloned();
        match identity_id {
            Some(identity_id) => (
                StatusCode::OK,
                Json(ValidatePayload {
                    session_id: format!("sess-{identity_id}"),
                    identity_id,
                    expires_at: "2030-01-01T00:00:00Z".to_string(),
                }),
            ),
            None => (
                StatusCode::UNAUTHORIZED,
                Json(ValidatePayload {
                    session_id: String::new(),
                    identity_id: String::new(),
                    expires_at: String::new(),
                }),
            ),
        }
    }

    async fn watchers_endpoint(
        Path(identity_id): Path<String>,
        State(state): State<PresenceApiStubState>,
        headers: HeaderMap,
    ) -> (StatusCode, Json<Value>) {
        let token_valid = headers
            .get("x-hexrelay-internal-token")
            .and_then(|value| value.to_str().ok())
            .map(|value| value == state.internal_token)
            .unwrap_or(false);
        if !token_valid {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "watchers": [] })),
            );
        }

        let watchers = state
            .watchers
            .read()
            .await
            .get(&identity_id)
            .cloned()
            .unwrap_or_default();
        (
            StatusCode::OK,
            Json(serde_json::json!({ "watchers": watchers })),
        )
    }

    let state = PresenceApiStubState {
        sessions: Arc::new(RwLock::new(sessions)),
        watchers: Arc::new(RwLock::new(watchers)),
        internal_token: internal_token.to_string(),
    };
    let app = Router::new()
        .route("/v1/auth/sessions/validate", get(validate_endpoint))
        .route(
            "/v1/internal/presence/watchers/:identity_id",
            get(watchers_endpoint),
        )
        .with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind presence stub listener");
    let address = listener.local_addr().expect("read presence stub address");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve presence stub API");
    });

    format!("http://{}", address)
}

async fn prepared_redis_client() -> Option<redis::Client> {
    let redis_url = match env::var("REALTIME_PRESENCE_REDIS_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            assert!(
                env::var("GITHUB_ACTIONS").is_err(),
                "REALTIME_PRESENCE_REDIS_URL must be set in GitHub Actions"
            );
            eprintln!(
                "[realtime-rs test] skipping Redis-backed presence test because REALTIME_PRESENCE_REDIS_URL is not configured"
            );
            return None;
        }
    };

    let client = match redis::Client::open(redis_url.as_str()) {
        Ok(value) => value,
        Err(error) => {
            assert!(
                env::var("GITHUB_ACTIONS").is_err(),
                "invalid Redis URL in GitHub Actions: {error}"
            );
            eprintln!("[realtime-rs test] skipping Redis-backed presence test because Redis URL is invalid: {error}");
            return None;
        }
    };

    let mut connection = match client.get_multiplexed_tokio_connection().await {
        Ok(value) => value,
        Err(error) => {
            assert!(
                env::var("GITHUB_ACTIONS").is_err(),
                "failed to connect to Redis in GitHub Actions: {error}"
            );
            eprintln!("[realtime-rs test] skipping Redis-backed presence test because Redis is unavailable: {error}");
            return None;
        }
    };

    let _: String = redis::cmd("PING")
        .query_async(&mut connection)
        .await
        .expect("ping Redis");

    Some(client)
}

async fn clear_presence_keys(client: &redis::Client, identity_id: &str) {
    let mut connection = client
        .get_multiplexed_tokio_connection()
        .await
        .expect("open redis connection");
    let _: () = redis::cmd("DEL")
        .arg(format!("presence:v1:count:{identity_id}"))
        .arg(format!("presence:v1:seq:{identity_id}"))
        .arg(format!("presence:v1:snapshot:{identity_id}"))
        .arg(format!("presence:v1:watcher_stream_head:{identity_id}"))
        .arg(format!("presence:v1:watcher_stream_log:{identity_id}"))
        .arg(format!(
            "presence:v1:watcher_device_cursor:{identity_id}:device-primary"
        ))
        .arg(format!(
            "presence:v1:watcher_device_cursor:{identity_id}:device-late"
        ))
        .query_async(&mut connection)
        .await
        .expect("clear presence keys");
}

async fn clear_channel_keys(client: &redis::Client, identity_id: &str) {
    let mut connection = client
        .get_multiplexed_tokio_connection()
        .await
        .expect("open redis connection");
    let _: () = redis::cmd("DEL")
        .arg(format!("channels:v1:recipient_stream_head:{identity_id}"))
        .arg(format!("channels:v1:recipient_stream_log:{identity_id}"))
        .arg(format!(
            "channels:v1:recipient_device_cursor:{identity_id}:device-primary"
        ))
        .arg(format!(
            "channels:v1:recipient_device_cursor:{identity_id}:device-late"
        ))
        .query_async(&mut connection)
        .await
        .expect("clear channel keys");
}

async fn connect_ws_with_token(
    ws_url: &str,
    token: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).expect("authorization header"),
    );
    set_allowed_origin(&mut request);

    let (socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");
    socket
}

async fn connect_ws_with_token_and_device(
    ws_url: &str,
    token: &str,
    device_id: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).expect("authorization header"),
    );
    request.headers_mut().insert(
        "x-hexrelay-device-id",
        HeaderValue::from_str(device_id).expect("device header"),
    );
    set_allowed_origin(&mut request);

    let (socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");
    socket
}

async fn recv_presence_event(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    expected_user_id: &str,
    expected_status: &str,
) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let message = tokio::time::timeout(remaining, socket.next())
            .await
            .expect("presence event timeout")
            .expect("socket message")
            .expect("ws frame");
        let text = match message {
            WsMessage::Text(value) => value,
            _ => continue,
        };
        let payload: Value = serde_json::from_str(&text).expect("decode websocket payload");
        if payload["event_type"] == "presence.updated"
            && payload["data"]["user_id"] == expected_user_id
            && payload["data"]["status"] == expected_status
        {
            return payload;
        }
    }
}

async fn close_socket_and_wait_for_disconnect(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(message) = socket.next().await {
            match message {
                Ok(WsMessage::Close(_)) | Err(_) => break,
                _ => continue,
            }
        }
    })
    .await;
}

async fn assert_no_presence_event(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    expected_user_id: &str,
    expected_status: &str,
    timeout: Duration,
) {
    let wait_result = tokio::time::timeout(timeout, async {
        while let Some(message) = socket.next().await {
            let message = match message {
                Ok(value) => value,
                Err(_) => return,
            };
            let text = match message {
                WsMessage::Text(value) => value,
                _ => continue,
            };
            let payload: Value = match serde_json::from_str(&text) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if payload["event_type"] == "presence.updated"
                && payload["data"]["user_id"] == expected_user_id
                && payload["data"]["status"] == expected_status
            {
                panic!(
                    "unexpected duplicate presence event for user={expected_user_id} status={expected_status}: {text}"
                );
            }
        }
    })
    .await;

    if let Ok(()) = wait_result {
        panic!("socket closed before absence assertion completed");
    }
}

#[tokio::test]
async fn rejects_missing_authorization_header() {
    let state = AppState::new(
        "http://127.0.0.1:1".to_string(),
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        None,
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build app state");
    let headers = HeaderMap::new();

    assert!(!is_session_valid(&state, &headers).await);
}

#[tokio::test]
async fn accepts_valid_authorization_with_successful_validation() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        None,
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build app state");
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );

    assert!(is_session_valid(&state, &headers).await);
}

#[tokio::test]
async fn rejects_authorization_when_validation_endpoint_denies() {
    let api_base = start_validate_server(ValidateMode::Denied).await;
    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        None,
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build app state");
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );

    assert!(!is_session_valid(&state, &headers).await);
}

async fn start_ws_server(api_base_url: String, ws_connect_rate_limit: usize) -> String {
    start_ws_server_with_limits(
        api_base_url,
        ws_connect_rate_limit,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .await
}

async fn start_ws_server_with_state(state: AppState) -> String {
    spawn_presence_subscriber(state.clone());
    spawn_channel_subscriber(state.clone());
    let app = build_app(state);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind websocket listener");
    let address = listener
        .local_addr()
        .expect("read websocket listener address");

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve websocket app");
    });

    format!("ws://{}/ws", address)
}

#[allow(clippy::too_many_arguments)]
async fn start_ws_server_with_limits(
    api_base_url: String,
    ws_connect_rate_limit: usize,
    ws_max_inbound_message_bytes: usize,
    ws_message_rate_limit: usize,
    ws_message_rate_window_seconds: u64,
    ws_max_connections_per_identity: usize,
    ws_auth_grace_seconds: u64,
    ws_auth_cache_max_entries: usize,
) -> String {
    let state = AppState::new(
        api_base_url,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        None,
        false,
        ws_connect_rate_limit,
        60,
        ws_max_inbound_message_bytes,
        ws_message_rate_limit,
        ws_message_rate_window_seconds,
        ws_max_connections_per_identity,
        ws_auth_grace_seconds,
        ws_auth_cache_max_entries,
    )
    .expect("build app state");

    start_ws_server_with_state(state).await
}

#[tokio::test]
async fn websocket_upgrade_rejects_missing_authorization() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    set_allowed_origin(&mut request);

    let result = connect_async(request).await;

    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("401") || message.contains("Unauthorized"));
}

#[tokio::test]
async fn websocket_upgrade_rejects_disallowed_origin() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    request
        .headers_mut()
        .insert("origin", HeaderValue::from_static("https://evil.example"));

    let result = connect_async(request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("403") || message.contains("Forbidden"));
}

#[tokio::test]
async fn websocket_upgrade_rejects_missing_origin() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );

    let result = connect_async(request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("403") || message.contains("Forbidden"));
}

#[tokio::test]
async fn websocket_upgrade_accepts_valid_cookie() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "cookie",
        HeaderValue::from_static("hexrelay_session=test-token"),
    );
    set_allowed_origin(&mut request);

    let connection = connect_async(request)
        .await
        .expect("websocket connect response");

    assert_eq!(connection.1.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn websocket_upgrade_accepts_valid_authorization() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let connection = connect_async(request)
        .await
        .expect("websocket connect response");

    assert_eq!(connection.1.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn websocket_presence_updates_propagate_and_recover_after_reconnect() {
    let Some(redis_client) = prepared_redis_client().await else {
        return;
    };

    clear_presence_keys(&redis_client, "usr-subject").await;
    clear_presence_keys(&redis_client, "usr-watcher").await;

    let api_base = start_presence_api_stub(
        HashMap::from([
            (
                "Bearer subject-token".to_string(),
                "usr-subject".to_string(),
            ),
            (
                "Bearer watcher-token".to_string(),
                "usr-watcher".to_string(),
            ),
        ]),
        HashMap::from([
            (
                "usr-subject".to_string(),
                vec!["usr-subject".to_string(), "usr-watcher".to_string()],
            ),
            ("usr-watcher".to_string(), vec!["usr-watcher".to_string()]),
        ]),
        "hexrelay-dev-presence-token-change-me",
    )
    .await;

    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        Some(redis_client.clone()),
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build app state");
    let ws_url = start_ws_server_with_state(state).await;

    let mut watcher_socket = connect_ws_with_token(&ws_url, "watcher-token").await;
    let _ = watcher_socket.next().await;
    let _ = recv_presence_event(&mut watcher_socket, "usr-watcher", "online").await;

    let mut subject_socket = connect_ws_with_token(&ws_url, "subject-token").await;
    let _ = subject_socket.next().await;

    let online_payload = recv_presence_event(&mut watcher_socket, "usr-subject", "online").await;
    let first_seq = online_payload["data"]["presence_seq"]
        .as_u64()
        .expect("online seq");

    subject_socket
        .close(None)
        .await
        .expect("close subject socket");
    let offline_payload = recv_presence_event(&mut watcher_socket, "usr-subject", "offline").await;
    let second_seq = offline_payload["data"]["presence_seq"]
        .as_u64()
        .expect("offline seq");
    assert!(second_seq > first_seq);

    let mut subject_socket = connect_ws_with_token(&ws_url, "subject-token").await;
    let _ = subject_socket.next().await;
    let reconnect_payload = recv_presence_event(&mut watcher_socket, "usr-subject", "online").await;
    let third_seq = reconnect_payload["data"]["presence_seq"]
        .as_u64()
        .expect("reconnect seq");
    assert!(third_seq > second_seq);

    subject_socket
        .close(None)
        .await
        .expect("close reconnected subject socket");
    watcher_socket
        .close(None)
        .await
        .expect("close watcher socket");
    clear_presence_keys(&redis_client, "usr-subject").await;
    clear_presence_keys(&redis_client, "usr-watcher").await;
}

#[tokio::test]
async fn websocket_presence_hydrates_late_profile_device_and_converges_live() {
    let Some(redis_client) = prepared_redis_client().await else {
        return;
    };

    clear_presence_keys(&redis_client, "usr-presence-subject").await;
    clear_presence_keys(&redis_client, "usr-presence-viewer").await;

    let api_base = start_presence_api_stub(
        HashMap::from([
            (
                "Bearer subject-token".to_string(),
                "usr-presence-subject".to_string(),
            ),
            (
                "Bearer viewer-token".to_string(),
                "usr-presence-viewer".to_string(),
            ),
        ]),
        HashMap::from([(
            "usr-presence-subject".to_string(),
            vec!["usr-presence-viewer".to_string()],
        )]),
        "hexrelay-dev-presence-token-change-me",
    )
    .await;

    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        Some(redis_client.clone()),
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build app state");
    let ws_url = start_ws_server_with_state(state).await;

    let mut primary_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-primary").await;
    let _ = primary_device.next().await;

    let mut subject_socket = connect_ws_with_token(&ws_url, "subject-token").await;
    let _ = subject_socket.next().await;

    let online_payload =
        recv_presence_event(&mut primary_device, "usr-presence-subject", "online").await;
    let online_seq = online_payload["data"]["presence_seq"]
        .as_u64()
        .expect("online seq");

    let mut late_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-late").await;
    let _ = late_device.next().await;

    let hydrated_payload =
        recv_presence_event(&mut late_device, "usr-presence-subject", "online").await;
    assert_eq!(hydrated_payload["data"]["presence_seq"], online_seq);

    subject_socket
        .close(None)
        .await
        .expect("close subject socket");

    let offline_primary =
        recv_presence_event(&mut primary_device, "usr-presence-subject", "offline").await;
    let offline_late =
        recv_presence_event(&mut late_device, "usr-presence-subject", "offline").await;
    assert!(
        offline_primary["data"]["presence_seq"]
            .as_u64()
            .expect("primary offline seq")
            > online_seq
    );
    assert_eq!(
        offline_primary["data"]["presence_seq"],
        offline_late["data"]["presence_seq"]
    );

    primary_device
        .close(None)
        .await
        .expect("close primary device");
    late_device.close(None).await.expect("close late device");
    clear_presence_keys(&redis_client, "usr-presence-subject").await;
    clear_presence_keys(&redis_client, "usr-presence-viewer").await;
}

#[tokio::test]
async fn websocket_presence_rehydrates_missed_offline_transition_for_reconnecting_device() {
    let Some(redis_client) = prepared_redis_client().await else {
        return;
    };

    clear_presence_keys(&redis_client, "usr-offline-subject").await;
    clear_presence_keys(&redis_client, "usr-offline-viewer").await;

    let api_base = start_presence_api_stub(
        HashMap::from([
            (
                "Bearer subject-token".to_string(),
                "usr-offline-subject".to_string(),
            ),
            (
                "Bearer viewer-token".to_string(),
                "usr-offline-viewer".to_string(),
            ),
        ]),
        HashMap::from([(
            "usr-offline-subject".to_string(),
            vec!["usr-offline-viewer".to_string()],
        )]),
        "hexrelay-dev-presence-token-change-me",
    )
    .await;

    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        Some(redis_client.clone()),
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build app state");
    let ws_url = start_ws_server_with_state(state).await;

    let mut primary_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-primary").await;
    let _ = primary_device.next().await;

    let mut late_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-late").await;
    let _ = late_device.next().await;

    let mut subject_socket = connect_ws_with_token(&ws_url, "subject-token").await;
    let _ = subject_socket.next().await;

    let online_primary =
        recv_presence_event(&mut primary_device, "usr-offline-subject", "online").await;
    let online_late = recv_presence_event(&mut late_device, "usr-offline-subject", "online").await;
    assert_eq!(
        online_primary["data"]["presence_seq"],
        online_late["data"]["presence_seq"]
    );

    late_device
        .close(None)
        .await
        .expect("close late device before offline transition");
    close_socket_and_wait_for_disconnect(&mut late_device).await;
    drop(late_device);

    subject_socket
        .close(None)
        .await
        .expect("close subject socket");

    let offline_primary =
        recv_presence_event(&mut primary_device, "usr-offline-subject", "offline").await;

    let mut reconnected_late_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-late").await;
    let _ = reconnected_late_device.next().await;

    let offline_rehydrated = recv_presence_event(
        &mut reconnected_late_device,
        "usr-offline-subject",
        "offline",
    )
    .await;
    assert_eq!(
        offline_primary["data"]["presence_seq"],
        offline_rehydrated["data"]["presence_seq"]
    );

    close_socket_and_wait_for_disconnect(&mut reconnected_late_device).await;
    let mut second_reconnect_late_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-late").await;
    let _ = second_reconnect_late_device.next().await;
    assert_no_presence_event(
        &mut second_reconnect_late_device,
        "usr-offline-subject",
        "offline",
        Duration::from_secs(2),
    )
    .await;

    primary_device
        .close(None)
        .await
        .expect("close primary device");
    second_reconnect_late_device
        .close(None)
        .await
        .expect("close second reconnected late device");
    clear_presence_keys(&redis_client, "usr-offline-subject").await;
    clear_presence_keys(&redis_client, "usr-offline-viewer").await;
}

#[tokio::test]
async fn websocket_channel_message_created_hydrates_late_profile_device() {
    let Some(redis_client) = prepared_redis_client().await else {
        return;
    };

    clear_channel_keys(&redis_client, "usr-channel-viewer").await;

    let api_base = start_presence_api_stub(
        HashMap::from([(
            "Bearer viewer-token".to_string(),
            "usr-channel-viewer".to_string(),
        )]),
        HashMap::new(),
        "hexrelay-dev-presence-token-change-me",
    )
    .await;

    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        Some(redis_client.clone()),
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build app state");
    let ws_url = start_ws_server_with_state(state.clone()).await;

    let mut primary_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-primary").await;
    let _ = primary_device.next().await;

    publish_channel_message_created(
        &state,
        PublishChannelMessageCreatedInput {
            message_id: "msg-1".to_string(),
            guild_id: "guild-1".to_string(),
            channel_id: "channel-1".to_string(),
            sender_id: "usr-sender".to_string(),
            created_at: Some("2026-03-23T05:00:00Z".to_string()),
            channel_seq: 7,
            recipients: vec!["usr-channel-viewer".to_string()],
        },
    )
    .await
    .expect("publish channel message created");

    let live_payload = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let message = primary_device
                .next()
                .await
                .expect("live channel event")
                .expect("ws frame");
            let text = match message {
                WsMessage::Text(value) => value,
                _ => continue,
            };
            let payload: Value = serde_json::from_str(&text).expect("decode channel payload");
            if payload["event_type"] == "channel.message.created"
                && payload["data"]["message_id"] == "msg-1"
            {
                break payload;
            }
        }
    })
    .await
    .expect("channel live timeout");
    assert_eq!(live_payload["data"]["channel_seq"], 7);

    let mut late_device =
        connect_ws_with_token_and_device(&ws_url, "viewer-token", "device-late").await;
    let _ = late_device.next().await;
    let hydrated_payload = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let message = late_device
                .next()
                .await
                .expect("hydrated channel event")
                .expect("ws frame");
            let text = match message {
                WsMessage::Text(value) => value,
                _ => continue,
            };
            let payload: Value = serde_json::from_str(&text).expect("decode hydrated payload");
            if payload["event_type"] == "channel.message.created"
                && payload["data"]["message_id"] == "msg-1"
            {
                break payload;
            }
        }
    })
    .await
    .expect("channel hydration timeout");
    assert_eq!(
        hydrated_payload["data"]["channel_seq"],
        live_payload["data"]["channel_seq"]
    );
    assert_eq!(
        hydrated_payload["data"]["created_at"],
        live_payload["data"]["created_at"]
    );

    primary_device
        .close(None)
        .await
        .expect("close primary device");
    late_device.close(None).await.expect("close late device");
    clear_channel_keys(&redis_client, "usr-channel-viewer").await;
}

#[tokio::test]
async fn websocket_replies_with_valid_event_envelope_for_self_targeted_call_signal_offer() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let (mut socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");

    let _ = socket.next().await;

    socket
        .send(WsMessage::Text(
            r#"{"event_type":"call.signal.offer","event_version":1,"correlation_id":"corr-123","data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-1","sdp_offer":"v=0\r\n"}}"#
                .to_string(),
        ))
        .await
        .expect("send offer event");

    let message = socket
        .next()
        .await
        .expect("socket message")
        .expect("ws frame");
    let text = match message {
        WsMessage::Text(value) => value,
        _ => panic!("expected text frame"),
    };

    let payload: Value = serde_json::from_str(&text).expect("decode response envelope");
    assert_eq!(payload["event_type"], "call.signal.offer");
    assert_eq!(payload["event_version"], 1);
    assert_eq!(payload["producer"], "realtime-gateway");
    assert_eq!(payload["correlation_id"], "corr-123");
    assert_eq!(payload["data"]["call_id"], "call-1");
}

#[tokio::test]
async fn websocket_rejects_cross_identity_call_signal_offer_until_fanout_exists() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let (mut socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");

    let _ = socket.next().await;

    socket
        .send(WsMessage::Text(
            r#"{"event_type":"call.signal.offer","event_version":1,"correlation_id":"corr-unsupported","data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-b","sdp_offer":"v=0\r\n"}}"#
                .to_string(),
        ))
        .await
        .expect("send offer event");

    let message = socket
        .next()
        .await
        .expect("socket message")
        .expect("ws frame");
    let text = match message {
        WsMessage::Text(value) => value,
        _ => panic!("expected text frame"),
    };

    let payload: Value = serde_json::from_str(&text).expect("decode response envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_unsupported");
}

#[test]
fn returns_error_for_invalid_event_payload() {
    let response = route_inbound_event(
        r#"{"event_type":"call.signal.offer","event_version":1,"data":{"call_id":"x"}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_invalid");
}

#[test]
fn returns_error_for_unsupported_version() {
    let response = route_inbound_event(
        r#"{"event_type":"call.signal.offer","event_version":2,"data":{}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_version_unsupported");
}

#[test]
fn returns_error_for_unsupported_event_type() {
    let response = route_inbound_event(
        r#"{"event_type":"presence.updated","event_version":1,"data":{}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_unsupported");
}

#[test]
fn returns_error_for_identity_mismatch() {
    let response = route_inbound_event(
        r#"{"event_type":"call.signal.offer","event_version":1,"data":{"call_id":"call-1","from_user_id":"usr-b","to_user_id":"usr-a","sdp_offer":"v=0\r\n"}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_identity_mismatch");
}

#[tokio::test]
async fn websocket_upgrade_rejects_when_api_is_unreachable() {
    let ws_url = start_ws_server("http://127.0.0.1:1".to_string(), 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let result = connect_async(request).await;
    assert!(result.is_err());

    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("401") || message.contains("Unauthorized"));
}

#[tokio::test]
async fn websocket_upgrade_accepts_cached_session_when_validation_is_unavailable() {
    let api_base = start_validate_server(ValidateMode::Unavailable).await;
    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        None,
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        30,
        10000,
    )
    .expect("build app state");

    let cache_key = auth_cache_key("Bearer test-token");
    state.validated_session_cache.lock().await.insert(
        cache_key,
        crate::state::CachedSession {
            identity_id: "usr-1".to_string(),
            expires_at: Utc::now() + ChronoDuration::minutes(5),
            validated_at: tokio::time::Instant::now(),
        },
    );

    let ws_url = start_ws_server_with_state(state).await;
    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let connection = connect_async(request)
        .await
        .expect("websocket connect response");
    assert_eq!(connection.1.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn websocket_upgrade_rejects_stale_cached_session_when_validation_is_unavailable() {
    let api_base = start_validate_server(ValidateMode::Unavailable).await;
    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        None,
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        30,
        10000,
    )
    .expect("build app state");

    let cache_key = auth_cache_key("Bearer test-token");
    state.validated_session_cache.lock().await.insert(
        cache_key,
        crate::state::CachedSession {
            identity_id: "usr-1".to_string(),
            expires_at: Utc::now() + ChronoDuration::minutes(5),
            validated_at: tokio::time::Instant::now() - Duration::from_secs(31),
        },
    );

    let ws_url = start_ws_server_with_state(state).await;
    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let result = connect_async(request).await;
    assert!(result.is_err());

    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("401") || message.contains("Unauthorized"));
}

#[tokio::test]
async fn websocket_upgrade_rejects_expired_cached_session_when_validation_is_unavailable() {
    let api_base = start_validate_server(ValidateMode::Unavailable).await;
    let state = AppState::new(
        api_base,
        test_allowed_origins(),
        "hexrelay-dev-presence-token-change-me".to_string(),
        None,
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        30,
        10000,
    )
    .expect("build app state");

    let cache_key = auth_cache_key("Bearer test-token");
    state.validated_session_cache.lock().await.insert(
        cache_key,
        crate::state::CachedSession {
            identity_id: "usr-1".to_string(),
            expires_at: Utc::now() - ChronoDuration::seconds(1),
            validated_at: tokio::time::Instant::now(),
        },
    );

    let ws_url = start_ws_server_with_state(state).await;
    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let result = connect_async(request).await;
    assert!(result.is_err());

    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("401") || message.contains("Unauthorized"));
}

#[tokio::test]
async fn websocket_upgrade_rejects_when_rate_limited() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 1).await;

    let mut first_request = ws_url
        .clone()
        .into_client_request()
        .expect("build first websocket request");
    first_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut first_request);

    let _ = connect_async(first_request)
        .await
        .expect("first websocket should connect");

    let mut second_request = ws_url
        .into_client_request()
        .expect("build second websocket request");
    second_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut second_request);

    let result = connect_async(second_request).await;
    assert!(result.is_err());

    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("429") || message.contains("Too Many Requests"));
}

#[tokio::test]
async fn websocket_upgrade_rate_limit_cannot_be_bypassed_by_rotating_auth_header() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server(api_base, 1).await;

    let mut first_request = ws_url
        .clone()
        .into_client_request()
        .expect("build first websocket request");
    first_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut first_request);

    let _ = connect_async(first_request)
        .await
        .expect("first websocket should connect");

    let mut second_request = ws_url
        .into_client_request()
        .expect("build second websocket request");
    second_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer totally-different-token"),
    );
    set_allowed_origin(&mut second_request);

    let result = connect_async(second_request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("429") || message.contains("Too Many Requests"));
}

#[tokio::test]
async fn websocket_closes_with_rate_limited_event_when_message_limit_exceeded() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server_with_limits(api_base, 60, 16384, 1, 60, 3, 0, 10000).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let (mut socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");

    let _ = socket.next().await;

    socket
        .send(WsMessage::Text(
            r#"{"event_type":"call.signal.offer","event_version":1,"correlation_id":"corr-1","data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-b","sdp_offer":"v=0\r\n"}}"#
                .to_string(),
        ))
        .await
        .expect("send first message");
    let _ = socket.next().await;

    socket
        .send(WsMessage::Text(
            r#"{"event_type":"call.signal.offer","event_version":1,"correlation_id":"corr-2","data":{"call_id":"call-2","from_user_id":"usr-1","to_user_id":"usr-c","sdp_offer":"v=0\r\n"}}"#
                .to_string(),
        ))
        .await
        .expect("send second message");

    let message = socket
        .next()
        .await
        .expect("socket message")
        .expect("ws frame");
    let text = match message {
        WsMessage::Text(value) => value,
        _ => panic!("expected text frame"),
    };

    let payload: Value = serde_json::from_str(&text).expect("decode error envelope");
    assert_eq!(payload["data"]["code"], "event_rate_limited");
}

#[tokio::test]
async fn websocket_upgrade_rejects_when_identity_connection_cap_exceeded() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server_with_limits(api_base, 60, 16384, 120, 60, 1, 0, 10000).await;

    let mut first_request = ws_url
        .clone()
        .into_client_request()
        .expect("build first websocket request");
    first_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut first_request);
    let (_first_socket, _) = connect_async(first_request)
        .await
        .expect("first websocket should connect");

    let mut second_request = ws_url
        .into_client_request()
        .expect("build second websocket request");
    second_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut second_request);

    let result = connect_async(second_request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("429") || message.contains("Too Many Requests"));
}

#[tokio::test]
async fn websocket_upgrade_allows_reconnect_after_connection_slot_release() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server_with_limits(api_base, 60, 16384, 120, 60, 1, 0, 10000).await;

    let mut first_request = ws_url
        .clone()
        .into_client_request()
        .expect("build first websocket request");
    first_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut first_request);
    let (mut first_socket, _) = connect_async(first_request)
        .await
        .expect("first websocket should connect");

    let mut second_request = ws_url
        .clone()
        .into_client_request()
        .expect("build second websocket request");
    second_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut second_request);

    let blocked = connect_async(second_request).await;
    assert!(blocked.is_err());

    first_socket.close(None).await.expect("close first socket");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut third_request = ws_url
        .into_client_request()
        .expect("build third websocket request");
    third_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut third_request);

    let reopened = connect_async(third_request)
        .await
        .expect("third websocket should connect after release");
    assert_eq!(reopened.1.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn websocket_rejects_text_payload_above_limit() {
    let api_base = start_validate_server(ValidateMode::Authorized).await;
    let ws_url = start_ws_server_with_limits(api_base, 60, 64, 120, 60, 3, 0, 10000).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let (mut socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");

    let _ = socket.next().await;

    socket
        .send(WsMessage::Text("x".repeat(1024)))
        .await
        .expect("send oversized payload");

    let message = socket
        .next()
        .await
        .expect("socket message")
        .expect("ws frame");

    let text = match message {
        WsMessage::Text(value) => value,
        _ => panic!("expected text frame"),
    };
    let payload: Value = serde_json::from_str(&text).expect("decode error envelope");
    assert_eq!(payload["data"]["code"], "event_too_large");
}
