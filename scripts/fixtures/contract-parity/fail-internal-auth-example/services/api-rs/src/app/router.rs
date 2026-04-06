fn router() {
    Router::new().route(
        "/v1/internal/presence/watchers/{identity_id}",
        get(list_presence_watchers),
    );
}
