use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures::stream::StreamExt;
use serde::Deserialize;

use crate::state::AppState;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ws_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if !is_session_valid(&state, &headers).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let _ = socket
        .send(Message::Text("realtime-rs connected".into()))
        .await;

    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let _ = socket.send(Message::Text(text)).await;
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

#[derive(Deserialize)]
struct SessionValidateResponse {
    #[serde(rename = "session_id")]
    _session_id: String,
    #[serde(rename = "identity_id")]
    _identity_id: String,
    #[serde(rename = "expires_at")]
    _expires_at: String,
}

async fn is_session_valid(state: &AppState, headers: &HeaderMap) -> bool {
    let authorization_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty());

    if authorization_header.is_none() {
        return false;
    }

    let url = format!("{}/v1/auth/sessions/validate", state.api_base_url);
    let mut request = state.http_client.get(url);
    if let Some(value) = authorization_header {
        request = request.header("authorization", value);
    }

    let response = match request.send().await {
        Ok(value) => value,
        Err(_) => return false,
    };

    if !response.status().is_success() {
        return false;
    }

    response.json::<SessionValidateResponse>().await.is_ok()
}

#[cfg(test)]
mod tests {
    use axum::{
        extract::State,
        http::{HeaderMap, HeaderValue, StatusCode},
        routing::get,
        Json, Router,
    };
    use serde::Serialize;
    use tokio::net::TcpListener;
    use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};

    use crate::{app::build_app, state::AppState};

    use super::is_session_valid;

    #[derive(Serialize)]
    struct ValidatePayload {
        session_id: String,
        identity_id: String,
        expires_at: String,
    }

    async fn start_validate_server(accept_authorization: bool) -> String {
        async fn validate_endpoint(
            State(accept_authorization): State<bool>,
            headers: HeaderMap,
        ) -> (StatusCode, Json<ValidatePayload>) {
            let authorized = headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(|value| value == "Bearer test-token")
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

            if accept_authorization {
                (StatusCode::OK, Json(payload))
            } else {
                (StatusCode::UNAUTHORIZED, Json(payload))
            }
        }

        let app = Router::new()
            .route("/v1/auth/sessions/validate", get(validate_endpoint))
            .with_state(accept_authorization);
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let address = listener.local_addr().expect("read listener address");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve test API");
        });

        format!("http://{}", address)
    }

    #[tokio::test]
    async fn rejects_missing_authorization_header() {
        let state = AppState::new("http://127.0.0.1:1".to_string());
        let headers = HeaderMap::new();

        assert!(!is_session_valid(&state, &headers).await);
    }

    #[tokio::test]
    async fn accepts_valid_authorization_with_successful_validation() {
        let api_base = start_validate_server(true).await;
        let state = AppState::new(api_base);
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test-token"),
        );

        assert!(is_session_valid(&state, &headers).await);
    }

    #[tokio::test]
    async fn rejects_authorization_when_validation_endpoint_denies() {
        let api_base = start_validate_server(false).await;
        let state = AppState::new(api_base);
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test-token"),
        );

        assert!(!is_session_valid(&state, &headers).await);
    }

    async fn start_ws_server(api_base_url: String) -> String {
        let app = build_app(AppState::new(api_base_url));
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind websocket listener");
        let address = listener
            .local_addr()
            .expect("read websocket listener address");

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve websocket app");
        });

        format!("ws://{}/ws", address)
    }

    #[tokio::test]
    async fn websocket_upgrade_rejects_missing_authorization() {
        let api_base = start_validate_server(true).await;
        let ws_url = start_ws_server(api_base).await;

        let result = connect_async(ws_url).await;

        assert!(result.is_err());
        let message = result
            .err()
            .map(|value| value.to_string())
            .unwrap_or_default();
        assert!(message.contains("401") || message.contains("Unauthorized"));
    }

    #[tokio::test]
    async fn websocket_upgrade_accepts_valid_authorization() {
        let api_base = start_validate_server(true).await;
        let ws_url = start_ws_server(api_base).await;

        let mut request = ws_url
            .into_client_request()
            .expect("build websocket client request");
        request.headers_mut().insert(
            "authorization",
            HeaderValue::from_static("Bearer test-token"),
        );

        let connection = connect_async(request)
            .await
            .expect("websocket connect response");

        assert_eq!(connection.1.status(), StatusCode::SWITCHING_PROTOCOLS);
    }
}
