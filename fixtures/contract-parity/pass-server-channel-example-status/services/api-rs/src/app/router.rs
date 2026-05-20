fn router() {
    Router::new().route(
        "/servers/:server_id/channels/:channel_id/messages",
        post(create_server_channel_message),
    );
}
