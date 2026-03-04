use axum::{extract::State, http::StatusCode, Json};
use tracing::info;

use crate::{
    errors::ApiResult,
    models::{HealthResponse, IdentityKeyRegistrationRequest, RegisteredIdentityKey},
    state::AppState,
    validation::validate_identity_registration,
};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

pub async fn register_identity_key(
    State(state): State<AppState>,
    Json(payload): Json<IdentityKeyRegistrationRequest>,
) -> ApiResult<StatusCode> {
    validate_identity_registration(&payload)?;

    let mut guard = state
        .identity_keys
        .write()
        .expect("acquire identity key write lock");

    let previous = guard.insert(
        payload.identity_id,
        RegisteredIdentityKey {
            public_key: payload.public_key,
            algorithm: payload.algorithm,
        },
    );

    if let Some(existing) = previous {
        info!(
            previous_algorithm = %existing.algorithm,
            previous_public_key_len = existing.public_key.len(),
            "replaced existing identity key registration"
        );
    }

    Ok(StatusCode::CREATED)
}
