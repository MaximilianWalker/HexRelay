fn router() {
    Router::new().route(
        "/internal/presence/watchers/{identity_id}",
        get(list_presence_watchers),
    );
}
