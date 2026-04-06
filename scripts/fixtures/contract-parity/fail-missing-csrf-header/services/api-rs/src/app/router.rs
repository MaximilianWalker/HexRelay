fn router() {
    Router::new().route(
        "/v1/friends/requests/{request_id}/accept",
        post(accept_friend_request),
    );
}
