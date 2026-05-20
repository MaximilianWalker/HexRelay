use axum::{extract::State, Json};

use crate::{
    models::{
        ServerAdministrationStatus, ServerAuthEndpoints, ServerCapabilitiesResponse,
        ServerConnectionResponse,
    },
    shared::errors::ApiResult,
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

const BASE_CAPABILITIES: &[&str] = &[
    "server_connect",
    "auth_session",
    "realtime_connect",
    "dm_encrypted_envelope",
];
const ADMIN_CAPABILITIES: &[&str] = &["server_admin", "server_operator"];
const OWNER_CAPABILITIES: &[&str] = &["server_owner"];

pub async fn get_server_connection(
    State(state): State<AppState>,
) -> ApiResult<Json<ServerConnectionResponse>> {
    Ok(Json(ServerConnectionResponse {
        service: "api-rs",
        server_id: effective_server_id(&state),
        server_public_key: effective_server_public_key(&state),
        runtime_api: "hexrelay_runtime_rest",
        auth_endpoints: ServerAuthEndpoints {
            challenge: "/auth/challenge",
            verify: "/auth/verify",
            session_validate: "/auth/sessions/validate",
        },
        capabilities_endpoint: "/server/capabilities",
    }))
}

pub async fn get_server_capabilities(
    auth: AuthSession,
    State(state): State<AppState>,
) -> ApiResult<Json<ServerCapabilitiesResponse>> {
    let is_server_owner = state.server_owner_identity_ids.contains(&auth.identity_id);
    let is_server_admin =
        is_server_owner || state.server_admin_identity_ids.contains(&auth.identity_id);

    let mut capabilities = BASE_CAPABILITIES.to_vec();
    if is_server_admin {
        capabilities.extend_from_slice(ADMIN_CAPABILITIES);
    }
    if is_server_owner {
        capabilities.extend_from_slice(OWNER_CAPABILITIES);
    }

    let mut scopes = Vec::new();
    if is_server_admin {
        scopes.extend_from_slice(ADMIN_CAPABILITIES);
    }
    if is_server_owner {
        scopes.extend_from_slice(OWNER_CAPABILITIES);
    }

    Ok(Json(ServerCapabilitiesResponse {
        server_id: effective_server_id(&state),
        server_public_key: effective_server_public_key(&state),
        identity_id: auth.identity_id,
        capabilities,
        administration: ServerAdministrationStatus {
            is_server_owner,
            is_server_admin,
            scopes,
        },
    }))
}

fn effective_server_id(state: &AppState) -> String {
    state
        .local_server_identity
        .as_ref()
        .map(|identity| identity.descriptor.server_id.clone())
        .unwrap_or_else(|| state.server_id.clone())
}

fn effective_server_public_key(state: &AppState) -> Option<String> {
    state
        .local_server_identity
        .as_ref()
        .map(|identity| identity.descriptor.server_public_key.clone())
}

#[cfg(test)]
mod tests {
    use super::effective_server_id;
    use crate::state::AppState;

    #[test]
    fn falls_back_to_server_id_when_no_local_descriptor_exists() {
        let state = AppState::default();
        assert_eq!(effective_server_id(&state), state.server_id);
    }
}
