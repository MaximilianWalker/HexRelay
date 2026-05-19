use axum::{extract::State, http::StatusCode, Json};

use crate::{
    app::state::AppState,
    models::{HealthResponse, ReadinessCheck, ReadinessResponse},
};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let mut checks = Vec::new();
    let mut ready = true;

    push_check(&mut checks, "database", database_ready(&state).await);
    push_check(
        &mut checks,
        "presence_redis",
        optional_redis_ready(&state).await,
    );

    for check in &checks {
        if check.status == "failed" {
            ready = false;
        }
    }

    let status = if ready { "ready" } else { "blocked" };
    let code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(ReadinessResponse {
            service: "api-rs",
            status,
            checks,
        }),
    )
}

fn push_check(
    checks: &mut Vec<ReadinessCheck>,
    name: &'static str,
    result: Result<Option<String>, String>,
) {
    match result {
        Ok(None) => checks.push(ReadinessCheck {
            name,
            status: "ok",
            detail: None,
        }),
        Ok(Some(detail)) => checks.push(ReadinessCheck {
            name,
            status: "skipped",
            detail: Some(detail),
        }),
        Err(detail) => checks.push(ReadinessCheck {
            name,
            status: "failed",
            detail: Some(detail),
        }),
    }
}

async fn database_ready(state: &AppState) -> Result<Option<String>, String> {
    let Some(pool) = state.db_pool.as_ref() else {
        return Err("database pool is not configured".to_string());
    };

    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| None)
        .map_err(|error| format!("database probe failed: {error}"))
}

async fn optional_redis_ready(state: &AppState) -> Result<Option<String>, String> {
    let Some(client) = state.presence_redis_client.as_ref() else {
        return Ok(Some("presence Redis is not configured".to_string()));
    };

    let mut connection = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|error| format!("open Redis connection: {error}"))?;
    let _: String = redis::cmd("PING")
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("Redis PING failed: {error}"))?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{health, ready};
    use crate::app::state::AppState;
    use axum::{extract::State, http::StatusCode};

    #[tokio::test]
    async fn returns_ok_health_payload() {
        let payload = health().await.0;
        assert_eq!(payload.service, "api-rs");
        assert_eq!(payload.status, "ok");
    }

    #[tokio::test]
    async fn readiness_blocks_when_database_pool_is_missing() {
        let (status, payload) = ready(State(AppState::default())).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload.0.service, "api-rs");
        assert_eq!(payload.0.status, "blocked");
        assert!(payload.0.checks.iter().any(|check| {
            check.name == "database"
                && check.status == "failed"
                && check
                    .detail
                    .as_deref()
                    .is_some_and(|detail| detail.contains("database pool is not configured"))
        }));
        assert!(payload
            .0
            .checks
            .iter()
            .any(|check| check.name == "presence_redis" && check.status == "skipped"));
    }
}
