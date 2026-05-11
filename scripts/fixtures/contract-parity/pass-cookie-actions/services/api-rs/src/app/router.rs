fn router() {
    Router::new()
        .route("/auth/verify", post(verify_auth_challenge))
        .route("/auth/sessions/revoke", post(revoke_session));
}
