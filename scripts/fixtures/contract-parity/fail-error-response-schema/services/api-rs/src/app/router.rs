use axum::{routing::post, Router};

use crate::transport::http::handlers::friends::accept_friend_request;

pub fn app_router() -> Router {
    Router::new().route(
        "/friends/requests/{request_id}/accept",
        post(accept_friend_request),
    )
}
