use axum::Json;

use crate::models::HealthResponse;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

#[cfg(test)]
mod tests {
    use super::health;

    #[tokio::test]
    async fn returns_ok_health_payload() {
        let payload = health().await.0;
        assert_eq!(payload.service, "api-rs");
        assert_eq!(payload.status, "ok");
    }
}
