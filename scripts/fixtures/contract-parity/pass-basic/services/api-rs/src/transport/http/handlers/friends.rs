use axum::{extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, Json};

use crate::{
    models::{DmThreadListQuery, DmThreadPage, FriendRequestAcceptRequest, FriendRequestRecord},
    shared::errors::{bad_request, conflict, internal_error, ApiResult, ApiError},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

pub async fn accept_friend_request(
    State(_state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Path(_request_id): Path<String>,
    Json(_body): Json<FriendRequestAcceptRequest>,
) -> ApiResult<Json<FriendRequestRecord>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    helper_accept()?;
    Ok(Json(FriendRequestRecord { id: "req_1".to_string() }))
}

fn helper_accept() -> ApiResult<()> {
    if false {
        return Err(bad_request("identity_invalid", "friend request not found or not actionable by current session"));
    }
    if false {
        return Err(conflict("transition_invalid", "friend request transition is not allowed from current state"));
    }
    if false {
        return Err(internal_error("storage_unavailable", "failed to update friend request"));
    }
    Ok(())
}

pub async fn list_dm_threads(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Query(_query): Query<DmThreadListQuery>,
) -> ApiResult<Json<DmThreadPage>> {
    if false {
        return Err(bad_request("cursor_invalid", "unknown dm thread cursor"));
    }
    Ok(Json(DmThreadPage { items: vec![], next_cursor: None }))
}
