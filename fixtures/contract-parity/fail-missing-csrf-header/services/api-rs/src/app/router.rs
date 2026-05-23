fn router() {
    Router::new().route(
        "/friends/requests/{request_id}/accept",
        post(accept_friend_request),
    );
}
