pub struct DmThreadListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub unread_only: Option<bool>,
}

pub struct DmThreadPage {
    pub items: Vec<String>,
    pub next_cursor: Option<String>,
}
