fn router() {
    Router::new().route(
        "/friends/requests/direct",
        post(create_friend_request_direct),
    );
}
