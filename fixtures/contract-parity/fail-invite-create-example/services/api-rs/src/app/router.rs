fn router() {
    Router::new().route("/invites", post(create_invite));
}
