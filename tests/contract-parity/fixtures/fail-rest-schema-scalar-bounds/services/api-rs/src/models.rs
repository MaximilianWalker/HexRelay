pub struct FriendRequestAcceptRequest {
    pub reason: Option<String>,
}

pub struct FriendRequestRecord {
    pub id: String,
}

pub struct DmThreadListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub unread_only: Option<bool>,
}

pub struct DmThreadPage {
    pub items: Vec<String>,
    pub next_cursor: Option<String>,
}

pub struct DmFanoutCatchUpRequest {
    pub device_id: String,
    pub device_secret: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

pub struct DmFanoutCatchUpItem {
    pub envelope_id: String,
    pub cursor: String,
    pub thread_id: String,
    pub message_id: String,
    pub ciphertext: String,
    pub source_device_id: Option<String>,
}

pub struct DmFanoutCatchUpResponse {
    pub status: String,
    pub reason_code: String,
    pub transport_profile: String,
    pub device_id: String,
    pub replay_count: u32,
    pub next_cursor: String,
    pub deduped_message_ids: Vec<String>,
    pub items: Vec<DmFanoutCatchUpItem>,
}
