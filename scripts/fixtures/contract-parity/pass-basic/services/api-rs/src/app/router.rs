fn router() {
    Router::new()
        .route(
            "/friends/requests/{request_id}/accept",
            post(accept_friend_request),
        )
        .route("/internal/raw-forward", post(forward_raw_body))
        .route("/dm/threads", get(list_dm_threads));
}
