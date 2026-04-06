pub struct ApiError {
    pub code: &'static str,
    pub message: &'static str,
}

pub type ApiResult<T> = Result<T, (axum::http::StatusCode, axum::Json<ApiError>)>;

pub fn bad_request(
    code: &'static str,
    message: &'static str,
) -> (axum::http::StatusCode, axum::Json<ApiError>) {
    (
        axum::http::StatusCode::BAD_REQUEST,
        axum::Json(ApiError { code, message }),
    )
}

pub fn conflict(
    code: &'static str,
    message: &'static str,
) -> (axum::http::StatusCode, axum::Json<ApiError>) {
    (
        axum::http::StatusCode::CONFLICT,
        axum::Json(ApiError { code, message }),
    )
}

pub fn forbidden(
    code: &'static str,
    message: &'static str,
) -> (axum::http::StatusCode, axum::Json<ApiError>) {
    (
        axum::http::StatusCode::FORBIDDEN,
        axum::Json(ApiError { code, message }),
    )
}

pub fn internal_error(
    code: &'static str,
    message: &'static str,
) -> (axum::http::StatusCode, axum::Json<ApiError>) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(ApiError { code, message }),
    )
}
