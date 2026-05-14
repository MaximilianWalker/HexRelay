fn router() {
    Router::new().route("/health", get(health));
}
