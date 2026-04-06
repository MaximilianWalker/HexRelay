fn router() {
    Router::new().route("/v1/dm/fanout/dispatch", post(run_dm_active_fanout));
}
