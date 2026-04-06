fn router() {
    Router::new().route(
        "/v1/friends/requests/direct",
        post(create_friend_request_direct),
    );
}
