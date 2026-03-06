use axum::{routing::get, Router};

use crate::{
    app::state::AppState,
    transport::ws::handlers::{health, ws_handler},
};

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .with_state(state)
}
