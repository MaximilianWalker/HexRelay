use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::{health, register_identity_key},
    state::AppState,
};

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/identity/keys/register", post(register_identity_key))
        .with_state(state)
}
