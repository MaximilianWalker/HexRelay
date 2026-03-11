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

#[cfg(test)]
mod tests {
    use super::{validate_friend_request_create, validate_friend_request_list_query};
    use crate::models::{FriendRequestCreate, FriendRequestListQuery};

    #[test]
    fn rejects_same_requester_and_target() {
        let payload = FriendRequestCreate {
            requester_identity_id: "usr-a".to_string(),
            target_identity_id: "usr-a".to_string(),
        };

        let err = validate_friend_request_create(&payload).expect_err("same identity must fail");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_invalid_direction_query() {
        let query = FriendRequestListQuery {
            identity_id: "usr-valid".to_string(),
            direction: Some("sideways".to_string()),
        };

        let err = validate_friend_request_list_query(&query).expect_err("invalid direction");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn accepts_valid_direction_query() {
        let inbound = FriendRequestListQuery {
            identity_id: "usr-valid".to_string(),
            direction: Some("inbound".to_string()),
        };
        let outbound = FriendRequestListQuery {
            identity_id: "usr-valid".to_string(),
            direction: Some("outbound".to_string()),
        };

        assert!(validate_friend_request_list_query(&inbound).is_ok());
        assert!(validate_friend_request_list_query(&outbound).is_ok());
    }
}
