use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

use crate::{
    app::state::AppState,
    transport::ws::handlers::{health, ws_handler},
};

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(ws_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
