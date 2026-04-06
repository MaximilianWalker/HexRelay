fn router() {
    Router::new().route("/v1/dm/privacy-policy", post(update_dm_policy));
}
