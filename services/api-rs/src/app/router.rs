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
        dev_testing::{activate_testing_session, list_testing_profiles},
        directory::{
            block_remove_contact, create_server, get_server, join_server, leave_server,
            list_contacts, list_servers, update_contact_preferences, update_server_preferences,
        },
        discovery::list_discovery_users,
        dm::{
            ack_dm_envelope_internal, forward_dm_envelope_internal, get_dm_policy,
            heartbeat_dm_profile_device, list_dm_thread_messages, list_dm_threads,
            mark_dm_thread_read, run_dm_active_fanout, run_dm_fanout_catch_up, update_dm_policy,
            verify_dm_profile_device_internal,
        },
        friends::{
            accept_friend_request, cancel_friend_request, create_friend_request,
            decline_friend_request, get_friend_request_bootstrap, list_friend_requests,
        },
        health::health,
        invites::{create_invite, redeem_invite},
        presence::list_presence_watchers,
        server::{get_server_capabilities, get_server_connection},
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
        .route("/server/connection", get(get_server_connection))
        .route("/server/capabilities", get(get_server_capabilities))
        .route("/identity/keys/register", post(register_identity_key))
        .route("/auth/challenge", post(issue_auth_challenge))
        .route("/auth/verify", post(verify_auth_challenge))
        .route("/auth/sessions/revoke", post(revoke_session))
        .route("/auth/sessions/validate", get(validate_session))
        .route("/dev/testing/profiles", get(list_testing_profiles))
        .route("/dev/testing/sessions", post(activate_testing_session))
        .route("/invites", post(create_invite))
        .route("/invites/redeem", post(redeem_invite))
        .route("/servers", get(list_servers).post(create_server))
        .route("/servers/join", post(join_server))
        .route("/servers/:server_id", get(get_server))
        .route(
            "/servers/:server_id/preferences",
            patch(update_server_preferences),
        )
        .route("/servers/:server_id/leave", post(leave_server))
        .route("/servers/:server_id/channels", get(list_server_channels))
        .route(
            "/servers/:server_id/channels/:channel_id/messages",
            get(list_server_channel_messages).post(create_server_channel_message),
        )
        .route(
            "/servers/:server_id/channels/:channel_id/messages/:message_id",
            patch(edit_server_channel_message).delete(soft_delete_server_channel_message),
        )
        .route("/contacts", get(list_contacts))
        .route(
            "/contacts/:identity_id/preferences",
            patch(update_contact_preferences),
        )
        .route(
            "/contacts/:identity_id/block-remove",
            post(block_remove_contact),
        )
        .route(
            "/internal/presence/watchers/:identity_id",
            get(list_presence_watchers),
        )
        .route("/discovery/users", get(list_discovery_users))
        .route(
            "/dm/privacy-policy",
            get(get_dm_policy).post(update_dm_policy),
        )
        .route(
            "/dm/profile-devices/heartbeat",
            post(heartbeat_dm_profile_device),
        )
        .route("/dm/fanout/dispatch", post(run_dm_active_fanout))
        .route("/dm/fanout/catch-up", post(run_dm_fanout_catch_up))
        .route("/internal/dm/envelopes/ack", post(ack_dm_envelope_internal))
        .route(
            "/internal/dm/envelopes/forward",
            post(forward_dm_envelope_internal),
        )
        .route(
            "/internal/dm/profile-devices/verify",
            post(verify_dm_profile_device_internal),
        )
        .route("/dm/threads", get(list_dm_threads))
        .route(
            "/dm/threads/:thread_id/messages",
            get(list_dm_thread_messages),
        )
        .route("/dm/threads/:thread_id/read", post(mark_dm_thread_read))
        .route(
            "/friends/requests",
            post(create_friend_request).get(list_friend_requests),
        )
        .route(
            "/friends/requests/:request_id/accept",
            post(accept_friend_request),
        )
        .route(
            "/friends/requests/:request_id/decline",
            post(decline_friend_request),
        )
        .route(
            "/friends/requests/:request_id/cancel",
            post(cancel_friend_request),
        )
        .route(
            "/friends/requests/:request_id/bootstrap",
            get(get_friend_request_bootstrap),
        )
        .route("/users/block", post(block_user))
        .route("/users/unblock", post(unblock_user))
        .route("/users/blocked", get(list_blocked_users))
        .route("/users/mute", post(mute_user))
        .route("/users/unmute", post(unmute_user))
        .route("/users/muted", get(list_muted_users))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
