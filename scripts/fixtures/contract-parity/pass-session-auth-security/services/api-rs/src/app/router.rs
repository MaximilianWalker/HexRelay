fn router() {
    Router::new().route("/v1/dm/threads", get(list_dm_threads));
}
