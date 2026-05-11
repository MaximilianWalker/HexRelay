fn router() {
    Router::new().route("/server-messages", get(list_server_messages));
}
