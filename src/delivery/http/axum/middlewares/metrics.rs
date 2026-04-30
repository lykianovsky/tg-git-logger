use crate::infrastructure::metrics::registry::METRICS;
use axum::body::Body;
use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

pub async fn http_metrics_middleware(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().as_str().to_string();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    let response = next.run(req).await;
    let elapsed = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    METRICS
        .http_requests_total
        .with_label_values(&[&method, &route, &status])
        .inc();
    METRICS
        .http_request_duration_seconds
        .with_label_values(&[&method, &route])
        .observe(elapsed);

    response
}
