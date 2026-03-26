use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{
    app::state::AppState,
    transport::http::internal::{
        publish_channel_message_created_internal, publish_channel_message_deleted_internal,
        publish_channel_message_updated_internal,
    },
    transport::ws::handlers::{health, ws_handler},
};

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/internal/channels/messages/created",
            post(publish_channel_message_created_internal),
        )
        .route(
            "/internal/channels/messages/updated",
            post(publish_channel_message_updated_internal),
        )
        .route(
            "/internal/channels/messages/deleted",
            post(publish_channel_message_deleted_internal),
        )
        .route("/ws", get(ws_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
