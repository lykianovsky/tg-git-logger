use crate::infrastructure::metrics::registry::METRICS;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;

pub struct AxumMetricsController;

impl AxumMetricsController {
    pub async fn handle_get() -> impl IntoResponse {
        let body = METRICS.render();
        (
            StatusCode::OK,
            [(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
            body,
        )
    }
}
