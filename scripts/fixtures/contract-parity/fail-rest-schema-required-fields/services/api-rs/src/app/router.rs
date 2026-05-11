use axum::{routing::post, Router};

use crate::transport::http::handlers::auth::verify_auth;

pub fn app_router() -> Router {
    Router::new().route("/auth/verify", post(verify_auth))
}
