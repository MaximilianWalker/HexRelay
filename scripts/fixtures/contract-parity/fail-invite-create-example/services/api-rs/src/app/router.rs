fn router() {
    Router::new().route("/v1/invites", post(create_invite));
}
