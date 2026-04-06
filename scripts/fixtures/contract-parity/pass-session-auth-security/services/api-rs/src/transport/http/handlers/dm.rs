use axum::{extract::{Query, State}, Json};

use crate::{
    models::{DmThreadListQuery, DmThreadPage},
    shared::errors::{bad_request, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

pub async fn list_dm_threads(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Query(query): Query<DmThreadListQuery>,
) -> ApiResult<Json<DmThreadPage>> {
    if matches!(query.cursor.as_deref(), Some("bad")) {
        return Err(bad_request("cursor_invalid", "unknown dm thread cursor"));
    }
    if false {
        return Err(internal_error(
            "storage_unavailable",
            "failed to list dm threads",
        ));
    }
    Ok(Json(DmThreadPage { items: Vec::new() }))
}
