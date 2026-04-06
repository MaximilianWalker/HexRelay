pub struct FriendRequestCreateRequest {
    pub requester_identity_id: String,
    pub target_identity_id: String,
}

pub struct FriendRequestRecord {
    pub request_id: String,
}
