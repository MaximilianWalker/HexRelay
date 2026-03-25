use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    Json,
};

use crate::{
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

#[async_trait]
impl<S> FromRequestParts<S> for AuthorizedServerMembership
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<ApiError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(server_id) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| forbidden("server_access_denied", "server membership required"))?;
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
                .map_err(|_| {
                    internal_error("storage_unavailable", "failed to verify server membership")
                })?;

        if !is_member {
            return Err(forbidden(
                "server_access_denied",
                "server membership required",
            ));
        }

        parts.extensions.insert(auth.clone());

        Ok(Self {
            server_id,
            identity_id: auth.identity_id,
        })
    }
}
