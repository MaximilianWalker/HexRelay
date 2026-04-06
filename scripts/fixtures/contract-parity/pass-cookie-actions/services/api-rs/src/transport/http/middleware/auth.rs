pub struct AuthSession {
    pub identity_id: String,
}

pub fn csrf_cookie_name() -> &'static str {
    "hexrelay_csrf"
}

pub fn session_cookie_name() -> &'static str {
    "hexrelay_session"
}

pub fn enforce_csrf_for_cookie_auth<T>(
    _auth: &AuthSession,
    _headers: &T,
) -> Result<
    (),
    (
        axum::http::StatusCode,
        axum::Json<crate::shared::errors::ApiError>,
    ),
> {
    Ok(())
}
