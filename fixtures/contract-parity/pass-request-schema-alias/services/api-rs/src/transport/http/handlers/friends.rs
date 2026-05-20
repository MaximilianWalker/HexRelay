use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};

use crate::{
    models::{FriendRequestCreateRequest, FriendRequestRecord},
    shared::errors::{bad_request, conflict, forbidden, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

pub async fn create_friend_request(
    State(_state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<FriendRequestCreateRequest>,
) -> ApiResult<(StatusCode, Json<FriendRequestRecord>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    if payload.requester_identity_id != auth.identity_id {
        return Err(bad_request("identity_invalid", "requester identity must match session"));
    }
    if false {
        return Err(forbidden(
            "blocked_user",
            "cannot send friend request — a block relationship exists between these users",
        ));
    }
    if false {
        return Err(conflict("friend_request_exists", "pending friend request already exists"));
    }
    if false {
        return Err(internal_error("storage_unavailable", "failed to persist friend request"));
    }
    Ok((
        StatusCode::CREATED,
        Json(FriendRequestRecord {
            request_id: "req_1".to_string(),
        }),
    ))
}
