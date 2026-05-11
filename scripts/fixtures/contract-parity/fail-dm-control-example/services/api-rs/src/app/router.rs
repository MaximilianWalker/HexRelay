fn router() {
    Router::new().route("/dm/privacy-policy", post(update_dm_policy));
}
