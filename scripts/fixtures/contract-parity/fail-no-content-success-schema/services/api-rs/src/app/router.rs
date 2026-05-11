fn router() {
    Router::new().route("/auth/sessions/revoke", post(revoke_session));
}
