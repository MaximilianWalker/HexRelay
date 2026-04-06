pub struct ServerChannelMessageCreateRequest {
    pub content: String,
    pub reply_to_message_id: Option<String>,
    pub mention_identity_ids: Option<Vec<String>>,
}

pub struct ServerChannelMessageRecord {
    pub message_id: String,
}
