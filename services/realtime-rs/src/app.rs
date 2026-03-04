use axum::{routing::get, Router};

use crate::handlers::{health, ws_handler};
use crate::state::AppState;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .with_state(state)
}
