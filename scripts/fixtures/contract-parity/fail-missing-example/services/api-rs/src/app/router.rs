fn router() {
    Router::new().route(
        "/dm/threads/{thread_id}/messages",
        get(list_dm_thread_messages),
    );
}
