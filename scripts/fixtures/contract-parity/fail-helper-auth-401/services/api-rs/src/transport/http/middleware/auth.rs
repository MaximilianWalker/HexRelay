pub struct AuthSession {
    pub identity_id: String,
}

pub fn csrf_cookie_name() -> &'static str {
    "hexrelay_csrf"
}

pub fn session_cookie_name() -> &'static str {
    "hexrelay_session"
}
