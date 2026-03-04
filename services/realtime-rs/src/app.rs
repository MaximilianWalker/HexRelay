use axum::{routing::get, Router};

use crate::handlers::{health, ws_handler};

pub fn build_app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
}
