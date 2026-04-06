pub type ApiResult<T> = Result<T, (axum::http::StatusCode, axum::Json<ApiError>)>;

pub struct ApiError {
    pub code: &'static str,
    pub message: &'static str,
}
