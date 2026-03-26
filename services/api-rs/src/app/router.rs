use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    routing::{get, patch, post},
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
        block_mute::{
            block_user, list_blocked_users, list_muted_users, mute_user, unblock_user, unmute_user,
        },
        directory::{get_server, list_contacts, list_servers},
        discovery::list_discovery_users,
        dm::{
            announce_dm_lan_discovery, create_dm_pairing_envelope, dm_connectivity_preflight,
            get_dm_policy, heartbeat_dm_profile_device, import_dm_pairing_envelope,
            list_dm_lan_peers, list_dm_thread_messages, list_dm_threads, mark_dm_thread_read,
            register_dm_endpoint_cards, revoke_dm_endpoint_cards, run_dm_active_fanout,
            run_dm_fanout_catch_up, run_dm_parallel_dial, run_dm_wan_wizard, update_dm_policy,
        },
        friends::{
            accept_friend_request, cancel_friend_request, create_friend_request,
            decline_friend_request, get_friend_request_bootstrap, list_friend_requests,
        },
        health::health,
        invites::{create_contact_invite, create_invite, redeem_contact_invite, redeem_invite},
        presence::list_presence_watchers,
        server_channels::{
            create_server_channel_message, edit_server_channel_message,
            list_server_channel_messages, list_server_channels, soft_delete_server_channel_message,
        },
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
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
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
        .route("/v1/servers/:server_id", get(get_server))
        .route("/v1/servers/:server_id/channels", get(list_server_channels))
        .route(
            "/v1/servers/:server_id/channels/:channel_id/messages",
            get(list_server_channel_messages).post(create_server_channel_message),
        )
        .route(
            "/v1/servers/:server_id/channels/:channel_id/messages/:message_id",
            patch(edit_server_channel_message).delete(soft_delete_server_channel_message),
        )
        .route("/v1/contacts", get(list_contacts))
        .route(
            "/v1/internal/presence/watchers/:identity_id",
            get(list_presence_watchers),
        )
        .route("/v1/discovery/users", get(list_discovery_users))
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
        .route(
            "/v1/dm/profile-devices/heartbeat",
            post(heartbeat_dm_profile_device),
        )
        .route("/v1/dm/fanout/dispatch", post(run_dm_active_fanout))
        .route("/v1/dm/fanout/catch-up", post(run_dm_fanout_catch_up))
        .route("/v1/dm/threads", get(list_dm_threads))
        .route(
            "/v1/dm/threads/:thread_id/messages",
            get(list_dm_thread_messages),
        )
        .route("/v1/dm/threads/:thread_id/read", post(mark_dm_thread_read))
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
        .route(
            "/v1/friends/requests/:request_id/bootstrap",
            get(get_friend_request_bootstrap),
        )
        .route("/v1/users/block", post(block_user))
        .route("/v1/users/unblock", post(unblock_user))
        .route("/v1/users/blocked", get(list_blocked_users))
        .route("/v1/users/mute", post(mute_user))
        .route("/v1/users/unmute", post(unmute_user))
        .route("/v1/users/muted", get(list_muted_users))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
