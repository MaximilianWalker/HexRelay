fn router() {
    Router::new().route("/auth/challenge", post(issue_auth_challenge));
}
