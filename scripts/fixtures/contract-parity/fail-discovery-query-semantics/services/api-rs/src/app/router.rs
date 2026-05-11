fn router() {
    Router::new().route("/discovery/users", get(list_discovery_users));
}
