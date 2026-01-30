use crate::applications::state::ApplicationState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::sync::Arc;

pub struct HealthHandler {}

impl HealthHandler {
    pub async fn handle(
        State(state): State<Arc<ApplicationState>>,
        headers: HeaderMap,
        Json(payload): Json<serde_json::Value>,
    ) -> StatusCode {
        StatusCode::NO_CONTENT
    }
}