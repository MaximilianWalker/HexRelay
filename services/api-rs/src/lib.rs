pub mod app;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod state;
pub mod validation;

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    use crate::{app::build_app, state::AppState};

    #[tokio::test]
    async fn registers_identity_key_with_hex_key() {
        let app = build_app(AppState::default());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-1","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"ed25519"}"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("get response");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn rejects_invalid_algorithm() {
        let app = build_app(AppState::default());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-1","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"rsa"}"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("get response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_invalid_public_key_format() {
        let app = build_app(AppState::default());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-1","public_key":"not-a-real-key","algorithm":"ed25519"}"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("get response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
