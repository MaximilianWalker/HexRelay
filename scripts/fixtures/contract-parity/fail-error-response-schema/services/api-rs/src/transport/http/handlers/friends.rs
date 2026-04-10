use axum::{Json, extract::{Path, State}};

use crate::{
    models::{FriendRequestAcceptRequest, FriendRequestRecord},
    shared::errors::{ApiResult, bad_request, conflict, internal_error},
};

pub async fn accept_friend_request(
    State(_state): State<()>,
    Path(_request_id): Path<String>,
    Json(_request): Json<FriendRequestAcceptRequest>,
) -> ApiResult<Json<FriendRequestRecord>> {
    if true {
        return Err(bad_request(
            "identity_invalid",
            "friend request not found or not actionable by current session",
        ));
    }
    if false {
        return Err(conflict(
            "transition_invalid",
            "friend request transition is not allowed from current state",
        ));
    }
    if false {
        return Err(internal_error("storage_unavailable", "failed to persist friend request transition"));
    }
    Ok(Json(FriendRequestRecord {
        request_id: "req-1".to_string(),
    }))
}
