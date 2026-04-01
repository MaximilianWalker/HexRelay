use std::collections::HashMap;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    Json,
};
use tracing::{debug, warn};

use crate::{
    infra::db::repos::server_channels_repo,
    infra::db::repos::servers_repo,
    models::ApiError,
    shared::errors::{forbidden, internal_error},
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedServerMembership {
    pub server_id: String,
    pub identity_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedServerChannelMembership {
    pub server_id: String,
    pub channel_id: String,
    pub identity_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerChannelAuthorizationFailure {
    ChannelNotFound,
    ServerAccessDenied,
}

impl From<ServerChannelAuthorizationFailure> for (StatusCode, Json<ApiError>) {
    fn from(value: ServerChannelAuthorizationFailure) -> Self {
        match value {
            ServerChannelAuthorizationFailure::ChannelNotFound => (
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    code: "channel_not_found",
                    message: "server channel was not found",
                }),
            ),
            ServerChannelAuthorizationFailure::ServerAccessDenied => {
                forbidden("server_access_denied", "server channel membership required")
            }
        }
    }
}

fn map_server_channel_authorization_failure(
    failure: ServerChannelAuthorizationFailure,
) -> (StatusCode, Json<ApiError>) {
    failure.into()
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthorizedServerMembership
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<ApiError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| forbidden("server_access_denied", "server membership required"))?;
        let server_id = params
            .get("server_id")
            .cloned()
            .ok_or_else(|| forbidden("server_access_denied", "server membership required"))?;
        let auth = AuthSession::from_request_parts(parts, state).await?;
        let app_state = AppState::from_ref(state);
        let pool = app_state.db_pool.as_ref().ok_or_else(|| {
            internal_error(
                "storage_unavailable",
                "server authorization requires configured database pool",
            )
        })?;

        let is_member =
            servers_repo::identity_has_server_membership(pool, &auth.identity_id, &server_id)
                .await
                .map_err(|error| {
                    warn!(
                        authorization_scope = "server_membership",
                        decision = "failure",
                        reason = "membership_lookup_failed",
                        identity_id = %auth.identity_id,
                        server_id = %server_id,
                        error = %error,
                        "server authorization lookup failed"
                    );
                    internal_error("storage_unavailable", "failed to verify server membership")
                })?;

        if !is_member {
            warn!(
                authorization_scope = "server_membership",
                decision = "deny",
                reason = "server_membership_required",
                identity_id = %auth.identity_id,
                server_id = %server_id,
                "server authorization denied"
            );
            return Err(forbidden(
                "server_access_denied",
                "server membership required",
            ));
        }

        debug!(
            authorization_scope = "server_membership",
            decision = "allow",
            identity_id = %auth.identity_id,
            server_id = %server_id,
            "server authorization allowed"
        );

        parts.extensions.insert(auth.clone());

        Ok(Self {
            server_id,
            identity_id: auth.identity_id,
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthorizedServerChannelMembership
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<ApiError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                map_server_channel_authorization_failure(
                    ServerChannelAuthorizationFailure::ServerAccessDenied,
                )
            })?;
        let server_id = params.get("server_id").cloned().ok_or_else(|| {
            map_server_channel_authorization_failure(
                ServerChannelAuthorizationFailure::ServerAccessDenied,
            )
        })?;
        let channel_id = params.get("channel_id").cloned().ok_or_else(|| {
            map_server_channel_authorization_failure(
                ServerChannelAuthorizationFailure::ServerAccessDenied,
            )
        })?;

        let app_state = AppState::from_ref(state);
        let pool = app_state.db_pool.as_ref().ok_or_else(|| {
            internal_error(
                "storage_unavailable",
                "server authorization requires configured database pool",
            )
        })?;

        let channel_id_exists = server_channels_repo::channel_id_exists(pool, &channel_id)
            .await
            .map_err(|error| {
                warn!(
                    authorization_scope = "server_channel",
                    decision = "failure",
                    reason = "channel_existence_lookup_failed",
                    server_id = %server_id,
                    channel_id = %channel_id,
                    error = %error,
                    "server channel authorization lookup failed"
                );
                internal_error("storage_unavailable", "failed to verify server channel")
            })?;

        if !channel_id_exists {
            warn!(
                authorization_scope = "server_channel",
                decision = "deny",
                reason = "channel_not_found",
                server_id = %server_id,
                channel_id = %channel_id,
                "server channel authorization denied"
            );
            return Err(map_server_channel_authorization_failure(
                ServerChannelAuthorizationFailure::ChannelNotFound,
            ));
        }

        let membership = AuthorizedServerMembership::from_request_parts(parts, state).await?;

        let channel_exists =
            server_channels_repo::server_channel_exists(pool, &membership.server_id, &channel_id)
                .await
                .map_err(|error| {
                    warn!(
                        authorization_scope = "server_channel",
                        decision = "failure",
                        reason = "server_channel_lookup_failed",
                        identity_id = %membership.identity_id,
                        server_id = %membership.server_id,
                        channel_id = %channel_id,
                        error = %error,
                        "server channel authorization lookup failed"
                    );
                    internal_error("storage_unavailable", "failed to verify server channel")
                })?;

        if !channel_exists {
            warn!(
                authorization_scope = "server_channel",
                decision = "deny",
                reason = "channel_not_in_server",
                identity_id = %membership.identity_id,
                server_id = %membership.server_id,
                channel_id = %channel_id,
                "server channel authorization denied"
            );
            return Err(map_server_channel_authorization_failure(
                ServerChannelAuthorizationFailure::ServerAccessDenied,
            ));
        }

        debug!(
            authorization_scope = "server_channel",
            decision = "allow",
            identity_id = %membership.identity_id,
            server_id = %membership.server_id,
            channel_id = %channel_id,
            "server channel authorization allowed"
        );

        Ok(Self {
            server_id: membership.server_id,
            channel_id,
            identity_id: membership.identity_id,
        })
    }
}
