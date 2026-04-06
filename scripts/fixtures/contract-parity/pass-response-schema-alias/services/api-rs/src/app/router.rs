fn router() {
    Router::new().route("/v1/server-messages", get(list_server_messages));
}
