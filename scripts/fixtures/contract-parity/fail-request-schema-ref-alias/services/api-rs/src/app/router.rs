fn router() {
    Router::new().route("/v1/friends/requests", post(create_friend_request));
}
