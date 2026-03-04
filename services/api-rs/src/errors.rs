use axum::{http::StatusCode, Json};

use crate::models::ApiError;

pub type ApiResult<T> = Result<T, (StatusCode, Json<ApiError>)>;

pub fn bad_request(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::BAD_REQUEST, Json(ApiError { code, message }))
}

pub fn unauthorized(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::UNAUTHORIZED, Json(ApiError { code, message }))
}

pub fn conflict(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::CONFLICT, Json(ApiError { code, message }))
}
