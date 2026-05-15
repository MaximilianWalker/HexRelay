use axum::Json;

use crate::{
    models::{DmFanoutCatchUpItem, DmFanoutCatchUpRequest, DmFanoutCatchUpResponse},
    shared::errors::ApiResult,
};

pub async fn catch_up_dm_fanout(
    Json(_payload): Json<DmFanoutCatchUpRequest>,
) -> ApiResult<Json<DmFanoutCatchUpResponse>> {
    Ok(Json(DmFanoutCatchUpResponse {
        status: "ready".to_string(),
        reason_code: "fanout_catch_up_ok".to_string(),
        transport_profile: "encrypted_envelope_node".to_string(),
        device_id: "dev-1".to_string(),
        replay_count: 1,
        next_cursor: "1".to_string(),
        deduped_message_ids: Vec::new(),
        items: vec![DmFanoutCatchUpItem {
            envelope_id: "env-1".to_string(),
            cursor: "1".to_string(),
            thread_id: "thread-1".to_string(),
            message_id: "msg-1".to_string(),
            ciphertext: "ciphertext".to_string(),
            source_device_id: None,
        }],
    }))
}
