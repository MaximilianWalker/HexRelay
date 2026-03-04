use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use std::{env, net::SocketAddr};
use tracing::info;

async fn health() -> &'static str {
    "ok"
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "realtime_rs=info,tower_http=info".to_string()),
        )
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    info!(%addr, "starting realtime service");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind realtime listener");
    axum::serve(listener, app)
        .await
        .expect("serve realtime application");
}
