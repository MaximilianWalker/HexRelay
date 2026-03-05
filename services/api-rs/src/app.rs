use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    auth_handlers::{
        issue_auth_challenge, register_identity_key, revoke_session, validate_session,
        verify_auth_challenge,
    },
    directory_handlers::{list_contacts, list_servers},
    handlers::{
        accept_friend_request, cancel_friend_request, create_friend_request, create_invite,
        decline_friend_request, health, list_friend_requests, redeem_invite,
    },
    state::AppState,
};

pub fn build_app(state: AppState) -> Router {
    let allowed_origins = state
        .allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect::<Vec<_>>();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true);

    Router::new()
        .route("/health", get(health))
        .route("/v1/identity/keys/register", post(register_identity_key))
        .route("/v1/auth/challenge", post(issue_auth_challenge))
        .route("/v1/auth/verify", post(verify_auth_challenge))
        .route("/v1/auth/sessions/revoke", post(revoke_session))
        .route("/v1/auth/sessions/validate", get(validate_session))
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
        .route(
            "/v1/friends/requests/:request_id/cancel",
            post(cancel_friend_request),
        )
        .layer(cors)
        .with_state(state)
}
