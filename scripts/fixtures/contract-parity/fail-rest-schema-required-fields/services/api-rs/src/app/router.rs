use axum::{routing::post, Router};

use crate::transport::http::handlers::auth::verify_auth;

pub fn app_router() -> Router {
    Router::new().route("/v1/auth/verify", post(verify_auth))
}
