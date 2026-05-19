use axum::{http::StatusCode, Json};

use crate::models::ApiError;

pub type ApiResult<T> = Result<T, (StatusCode, Json<ApiError>)>;

pub fn bad_request(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::BAD_REQUEST, Json(ApiError { code, message }))
}

pub fn unauthorized(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::UNAUTHORIZED, Json(ApiError { code, message }))
}

pub fn forbidden(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::FORBIDDEN, Json(ApiError { code, message }))
}

pub fn conflict(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::CONFLICT, Json(ApiError { code, message }))
}

pub fn too_many_requests(
    code: &'static str,
    message: &'static str,
) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(ApiError { code, message }),
    )
}

pub fn internal_error(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { code, message }),
    )
}

pub fn storage_error<E>(
    context: &'static str,
    code: &'static str,
    message: &'static str,
    error: E,
) -> (StatusCode, Json<ApiError>)
where
    E: std::fmt::Display,
{
    tracing::error!(
        storage_context = context,
        error = %error,
        "storage operation failed"
    );
    internal_error(code, message)
}

#[cfg(test)]
mod tests {
    use super::storage_error;
    use axum::{http::StatusCode, Json};

    #[test]
    fn storage_error_preserves_client_safe_response() {
        let (status, Json(error)) = storage_error(
            "test.storage",
            "storage_unavailable",
            "storage unavailable",
            "database timeout",
        );

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.code, "storage_unavailable");
        assert_eq!(error.message, "storage unavailable");
    }
}
