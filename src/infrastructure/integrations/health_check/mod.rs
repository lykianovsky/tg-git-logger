use crate::domain::health_ping::ports::health_check_client::{
    HealthCheckClient, HealthCheckResult,
};

pub struct ReqwestHealthCheckClient;

impl ReqwestHealthCheckClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl HealthCheckClient for ReqwestHealthCheckClient {
    async fn check(&self, url: &str) -> HealthCheckResult {
        let start = std::time::Instant::now();

        let result = reqwest::get(url).await;

        let response_ms = start.elapsed().as_millis() as i32;

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    HealthCheckResult {
                        is_success: true,
                        status_text: "ok".to_string(),
                        response_ms,
                        error_message: None,
                    }
                } else {
                    HealthCheckResult {
                        is_success: false,
                        status_text: "error".to_string(),
                        response_ms,
                        error_message: Some(format!("HTTP {}", response.status())),
                    }
                }
            }

            Err(e) => HealthCheckResult {
                is_success: false,
                status_text: "error".to_string(),
                response_ms,
                error_message: Some(e.to_string()),
            },
        }
    }
}
