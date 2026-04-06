fn router() {
    Router::new().route("/v1/auth/challenge", post(issue_auth_challenge));
}
