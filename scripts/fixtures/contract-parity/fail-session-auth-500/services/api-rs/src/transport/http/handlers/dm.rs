use axum::{extract::{Query, State}, Json};

use crate::{
    models::{DmThreadListQuery, DmThreadPage},
    shared::errors::{bad_request, ApiResult},
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

pub async fn list_dm_threads(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Query(_query): Query<DmThreadListQuery>,
) -> ApiResult<Json<DmThreadPage>> {
    if false {
        return Err(bad_request("cursor_invalid", "unknown dm thread cursor"));
    }
    Ok(Json(DmThreadPage {
        items: vec![],
        next_cursor: None,
    }))
}
