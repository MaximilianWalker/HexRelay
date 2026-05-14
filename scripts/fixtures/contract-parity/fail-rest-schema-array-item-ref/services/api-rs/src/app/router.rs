fn router() {
    Router::new()
        .route(
            "/friends/requests/{request_id}/accept",
            post(accept_friend_request),
        )
        .route("/dm/threads", get(list_dm_threads))
        .route("/dm/fanout/catch-up", post(catch_up_dm_fanout));
}
