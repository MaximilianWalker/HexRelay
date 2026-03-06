use crate::{
    models::{FriendRequestCreate, FriendRequestListQuery},
    shared::errors::{bad_request, ApiResult},
};

use crate::domain::auth::validation::is_valid_identity_id;

pub fn validate_friend_request_create(payload: &FriendRequestCreate) -> ApiResult<()> {
    if !is_valid_identity_id(&payload.requester_identity_id)
        || !is_valid_identity_id(&payload.target_identity_id)
    {
        return Err(bad_request(
            "identity_invalid",
            "requester_identity_id and target_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    if payload.requester_identity_id == payload.target_identity_id {
        return Err(bad_request(
            "identity_invalid",
            "requester and target must be different identities",
        ));
    }

    Ok(())
}

pub fn validate_friend_request_list_query(query: &FriendRequestListQuery) -> ApiResult<()> {
    if !is_valid_identity_id(&query.identity_id) {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    if let Some(direction) = query.direction.as_ref() {
        if direction != "inbound" && direction != "outbound" {
            return Err(bad_request(
                "identity_invalid",
                "direction must be inbound or outbound when provided",
            ));
        }
    }

    Ok(())
}
