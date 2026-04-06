fn router() {
    Router::new().route("/v1/auth/sessions/revoke", post(revoke_session));
}
