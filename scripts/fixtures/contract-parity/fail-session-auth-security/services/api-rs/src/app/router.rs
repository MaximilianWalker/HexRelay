fn router() {
    Router::new().route("/dm/threads", get(list_dm_threads));
}
