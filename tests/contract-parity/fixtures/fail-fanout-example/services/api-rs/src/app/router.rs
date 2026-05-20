fn router() {
    Router::new().route("/dm/fanout/dispatch", post(run_dm_active_fanout));
}
