use axum::{
    http::{header, Method},
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    handlers::{
        accept_friend_request, create_friend_request, create_invite, decline_friend_request,
        health, issue_auth_challenge, list_contacts, list_friend_requests, list_servers,
        redeem_invite, register_identity_key, revoke_session, verify_auth_challenge,
    },
    state::AppState,
};

pub fn build_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/health", get(health))
        .route("/v1/identity/keys/register", post(register_identity_key))
        .route("/v1/auth/challenge", post(issue_auth_challenge))
        .route("/v1/auth/verify", post(verify_auth_challenge))
        .route("/v1/auth/sessions/revoke", post(revoke_session))
        .route("/v1/invites", post(create_invite))
        .route("/v1/invites/redeem", post(redeem_invite))
        .route("/v1/servers", get(list_servers))
        .route("/v1/contacts", get(list_contacts))
        .route(
            "/v1/friends/requests",
            post(create_friend_request).get(list_friend_requests),
        )
        .route(
            "/v1/friends/requests/:request_id/accept",
            post(accept_friend_request),
        )
        .route(
            "/v1/friends/requests/:request_id/decline",
            post(decline_friend_request),
        )
        .layer(cors)
        .with_state(state)
}
