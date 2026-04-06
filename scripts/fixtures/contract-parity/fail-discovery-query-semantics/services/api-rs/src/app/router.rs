fn router() {
    Router::new().route("/v1/discovery/users", get(list_discovery_users));
}
