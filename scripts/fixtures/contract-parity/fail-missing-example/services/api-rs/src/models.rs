pub struct DmThreadMessageListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

pub struct DmMessagePage {
    pub items: Vec<String>,
    pub next_cursor: Option<String>,
}
