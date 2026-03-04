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
