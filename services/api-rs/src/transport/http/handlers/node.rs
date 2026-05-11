use axum::{extract::State, Json};

use crate::{
    models::{
        NodeAdministrationStatus, NodeAuthEndpoints, NodeCapabilitiesResponse,
        NodeConnectionResponse,
    },
    shared::errors::ApiResult,
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

const BASE_CAPABILITIES: &[&str] = &[
    "node_connect",
    "auth_session",
    "realtime_connect",
    "dm_encrypted_envelope",
];
const ADMIN_CAPABILITIES: &[&str] = &["node_admin", "node_operator"];
const OWNER_CAPABILITIES: &[&str] = &["node_owner"];

pub async fn get_node_connection(
    State(state): State<AppState>,
) -> ApiResult<Json<NodeConnectionResponse>> {
    Ok(Json(NodeConnectionResponse {
        service: "api-rs",
        node_id: effective_node_id(&state),
        node_fingerprint: state.node_fingerprint,
        runtime_api: "hexrelay_runtime_rest",
        auth_endpoints: NodeAuthEndpoints {
            challenge: "/auth/challenge",
            verify: "/auth/verify",
            session_validate: "/auth/sessions/validate",
        },
        capabilities_endpoint: "/node/capabilities",
    }))
}

pub async fn get_node_capabilities(
    auth: AuthSession,
    State(state): State<AppState>,
) -> ApiResult<Json<NodeCapabilitiesResponse>> {
    let is_node_owner = state.node_owner_identity_ids.contains(&auth.identity_id);
    let is_node_admin = is_node_owner || state.node_admin_identity_ids.contains(&auth.identity_id);

    let mut capabilities = BASE_CAPABILITIES.to_vec();
    if is_node_admin {
        capabilities.extend_from_slice(ADMIN_CAPABILITIES);
    }
    if is_node_owner {
        capabilities.extend_from_slice(OWNER_CAPABILITIES);
    }

    let mut scopes = Vec::new();
    if is_node_admin {
        scopes.extend_from_slice(ADMIN_CAPABILITIES);
    }
    if is_node_owner {
        scopes.extend_from_slice(OWNER_CAPABILITIES);
    }

    Ok(Json(NodeCapabilitiesResponse {
        node_id: effective_node_id(&state),
        node_fingerprint: state.node_fingerprint,
        identity_id: auth.identity_id,
        capabilities,
        administration: NodeAdministrationStatus {
            is_node_owner,
            is_node_admin,
            scopes,
        },
    }))
}

fn effective_node_id(state: &AppState) -> String {
    state
        .local_node_identity
        .as_ref()
        .map(|identity| identity.descriptor.node_id.clone())
        .unwrap_or_else(|| state.node_fingerprint.clone())
}

#[cfg(test)]
mod tests {
    use super::effective_node_id;
    use crate::state::AppState;

    #[test]
    fn falls_back_to_fingerprint_when_no_local_descriptor_exists() {
        let state = AppState::default();
        assert_eq!(effective_node_id(&state), state.node_fingerprint);
    }
}
