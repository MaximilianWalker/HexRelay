fn router() {
    Router::new().route("/friends/requests", post(create_friend_request));
}
