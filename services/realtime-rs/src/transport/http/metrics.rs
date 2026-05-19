use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
    response::IntoResponse,
};

use crate::state::AppState;

const PROMETHEUS_TEXT_CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static(PROMETHEUS_TEXT_CONTENT_TYPE),
    );

    (headers, state.metrics.render_prometheus())
}

#[cfg(test)]
mod tests {
    use super::metrics;
    use crate::{metrics::WebsocketUpgradeOutcome, state::AppState};
    use axum::{extract::State, http::header::CONTENT_TYPE, response::IntoResponse};

    #[tokio::test]
    async fn returns_prometheus_text_metrics() {
        let state = AppState::new(
            "http://127.0.0.1:8080".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
            "hexrelay-dev-presence-watcher-token-change-me".to_string(),
            None,
            false,
            120,
            60,
            64 * 1024,
            120,
            60,
            3,
            0,
            1024,
        )
        .expect("state");
        state
            .metrics
            .record_websocket_upgrade(WebsocketUpgradeOutcome::Accepted);

        let response = metrics(State(state)).await.into_response();
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "text/plain; version=0.0.4; charset=utf-8"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read metrics body");
        let body = String::from_utf8(body.to_vec()).expect("metrics utf8");
        assert!(body.contains("hexrelay_realtime_websocket_upgrade_total{outcome=\"accepted\"} 1"));
    }
}
