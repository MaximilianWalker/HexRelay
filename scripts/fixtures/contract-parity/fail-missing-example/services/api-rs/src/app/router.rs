fn router() {
    Router::new().route(
        "/v1/dm/threads/{thread_id}/messages",
        get(list_dm_thread_messages),
    );
}
