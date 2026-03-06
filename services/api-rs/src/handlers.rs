use axum::Json;

use crate::models::HealthResponse;

pub use crate::friend_request_handlers::{
    accept_friend_request, cancel_friend_request, create_friend_request, decline_friend_request,
    list_friend_requests,
};
pub use crate::invite_handlers::{create_invite, redeem_invite};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}
