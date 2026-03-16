use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    app::state::AppState,
    transport::http::handlers::{
        auth::{
            issue_auth_challenge, register_identity_key, revoke_session, validate_session,
            verify_auth_challenge,
        },
        directory::{list_contacts, list_servers},
        dm::{
            announce_dm_lan_discovery, create_dm_pairing_envelope, dm_connectivity_preflight,
            get_dm_policy, import_dm_pairing_envelope, list_dm_lan_peers, list_dm_thread_messages,
            list_dm_threads, register_dm_endpoint_cards, revoke_dm_endpoint_cards,
            run_dm_parallel_dial, run_dm_wan_wizard, update_dm_policy,
        },
        friends::{
            accept_friend_request, cancel_friend_request, create_friend_request,
            decline_friend_request, list_friend_requests,
        },
        health::health,
        invites::{create_contact_invite, create_invite, redeem_contact_invite, redeem_invite},
    },
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
        .route("/v1/contact-invites", post(create_contact_invite))
        .route("/v1/contact-invites/redeem", post(redeem_contact_invite))
        .route("/v1/servers", get(list_servers))
        .route("/v1/contacts", get(list_contacts))
        .route(
            "/v1/dm/privacy-policy",
            get(get_dm_policy).post(update_dm_policy),
        )
        .route("/v1/dm/pairing-envelope", post(create_dm_pairing_envelope))
        .route(
            "/v1/dm/pairing-envelope/import",
            post(import_dm_pairing_envelope),
        )
        .route(
            "/v1/dm/connectivity/preflight",
            post(dm_connectivity_preflight),
        )
        .route(
            "/v1/dm/connectivity/lan-discovery/announce",
            post(announce_dm_lan_discovery),
        )
        .route(
            "/v1/dm/connectivity/lan-discovery/peers",
            get(list_dm_lan_peers),
        )
        .route("/v1/dm/connectivity/wan-wizard", post(run_dm_wan_wizard))
        .route(
            "/v1/dm/connectivity/endpoint-cards",
            post(register_dm_endpoint_cards),
        )
        .route(
            "/v1/dm/connectivity/endpoint-cards/revoke",
            post(revoke_dm_endpoint_cards),
        )
        .route(
            "/v1/dm/connectivity/parallel-dial",
            post(run_dm_parallel_dial),
        )
        .route("/v1/dm/threads", get(list_dm_threads))
        .route(
            "/v1/dm/threads/:thread_id/messages",
            get(list_dm_thread_messages),
        )
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
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
