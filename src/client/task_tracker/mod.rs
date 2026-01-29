use reqwest::{Client, Method, Response};
use serde::Serialize;
use serde_json::json;

pub struct TaskTrackerClient {
    pub base: String,
    pub token: String,
    pub client: Client,
}

impl TaskTrackerClient {
    pub fn new(base: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            token: token.into(),
            client: Client::new(),
        }
    }

    async fn request<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<Response, reqwest::Error> {
        let url = format!("{}/api/latest{}", self.base, path);

        tracing::info!("{}", url);

        let mut req = self
            .client
            .request(method, &url)
            .bearer_auth(&self.token)
            .header("Content-Type", "application/json");

        if let Some(b) = body {
            req = req.json(b);
        }

        req.send().await
    }

    pub async fn move_card(&self, card_id: &str, column_id: &str) -> Result<(), reqwest::Error> {
        let body = json!({ "column_id": column_id });

        let resp = self
            .request(Method::PATCH, &format!("/cards/{}", card_id), Some(&body))
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            eprintln!("Failed to move card: {}", text);
        }

        Ok(())
    }
}
