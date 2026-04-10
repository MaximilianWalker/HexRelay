use axum::Json;

use crate::models::{AuthVerifyRequest, AuthVerifyResponse};

pub async fn verify_auth(Json(_request): Json<AuthVerifyRequest>) -> Json<AuthVerifyResponse> {
    Json(AuthVerifyResponse {
        session_id: "sess-1".to_string(),
        expires_at: "2030-01-01T00:00:00Z".to_string(),
    })
}
