use axum::Json;

use crate::{models::ServerChannelMessageRecord, shared::errors::ApiResult};

pub async fn list_server_messages() -> ApiResult<Json<ServerChannelMessageRecord>> {
    Ok(Json(ServerChannelMessageRecord {
        message_id: "msg_1".to_string(),
    }))
}
