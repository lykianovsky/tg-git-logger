pub struct HealthCheckResult {
    pub is_success: bool,
    pub status_text: String,
    pub response_ms: i32,
    pub error_message: Option<String>,
}

#[async_trait::async_trait]
pub trait HealthCheckClient: Send + Sync {
    async fn check(&self, url: &str) -> HealthCheckResult;
}
