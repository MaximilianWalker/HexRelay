fn router() {
    Router::new().route("/v1/auth/verify", post(verify_auth_challenge));
}
