use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct FriendRequestAcceptRequest {
    pub request_id: String,
}

#[derive(Serialize)]
pub struct FriendRequestRecord {
    pub request_id: String,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: &'static str,
}
