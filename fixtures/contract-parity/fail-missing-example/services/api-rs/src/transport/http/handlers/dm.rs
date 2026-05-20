use axum::{extract::{Path, Query, State}, http::StatusCode, Json};

use crate::{
    models::{DmMessagePage, DmThreadMessageListQuery},
    shared::errors::{bad_request, internal_error, ApiResult, ApiError},
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

pub async fn list_dm_thread_messages(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Path(_thread_id): Path<String>,
    Query(_query): Query<DmThreadMessageListQuery>,
) -> ApiResult<Json<DmMessagePage>> {
    if false {
        return Err(bad_request("cursor_invalid", "message cursor must be numeric"));
    }
    Err((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "thread_not_found",
            message: "dm thread was not found",
        }),
    ))
}
