use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::stream::StreamExt;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
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
