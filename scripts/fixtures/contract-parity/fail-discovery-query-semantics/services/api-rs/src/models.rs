pub struct DiscoveryUserListQuery {
    pub scope: Option<String>,
    pub query: Option<String>,
    pub limit: Option<u32>,
}

pub struct DiscoveryUserListResponse {
    pub items: Vec<String>,
}
