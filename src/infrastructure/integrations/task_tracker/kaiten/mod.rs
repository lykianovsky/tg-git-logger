use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::task::ports::task_tracker_client::{
    TaskTrackerClient, TaskTrackerClientMoveToColumnError,
};
use reqwest::{Client, Method};
use serde::Serialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct KaitenCard {
    pub column_id: u64,
}

pub struct KaitenClientBase(pub String);

pub struct KaitenClientToken(pub String);

pub struct KaitenClient {
    base: KaitenClientBase,
    token: KaitenClientToken,
    client: Client,
}

impl KaitenClient {
    pub fn new(base: KaitenClientBase, token: KaitenClientToken) -> Self {
        Self {
            base,
            token,
            client: Client::new(),
        }
    }

    async fn request<Body, Response>(
        &self,
        method: Method,
        path: &str,
        body: Option<&Body>,
    ) -> Result<Response, Box<dyn std::error::Error>>
    where
        Body: Serialize + ?Sized,
        Response: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/api/latest{}", self.base.0, path);

        tracing::info!("{}", url);

        let mut req = self
            .client
            .request(method, &url)
            .bearer_auth(&self.token.0)
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            tracing::debug!(
                "Request body: {}",
                serde_json::to_string(body).unwrap_or_default()
            );
            req = req.json(body);
        }

        let resp = req.send().await?;

        let status = resp.status();
        let text = resp.text().await?;

        tracing::debug!("Response status: {}", status);
        tracing::debug!("Response body: {}", text);

        let parsed = serde_json::from_str::<Response>(&text)?;

        Ok(parsed)
    }
}

#[async_trait]
impl TaskTrackerClient for KaitenClient {
    async fn move_task_to_column(
        &self,
        task_id: u64,
        column_id: u64,
    ) -> Result<(), TaskTrackerClientMoveToColumnError> {
        let body = json!({ "column_id": column_id });

        let response: KaitenCard = self
            .request(Method::PATCH, &format!("/cards/{}", task_id), Some(&body))
            .await
            .map_err(|e| TaskTrackerClientMoveToColumnError::ClientError(e.to_string()))?;

        let span = tracing::debug_span!(
            "move_card",
            task_id = task_id,
            column_id = column_id,
            response_column_id = response.column_id
        );
        let _enter = span.enter();

        tracing::debug!("MoveCard response: {:?}", response);

        if response.column_id != column_id {
            tracing::error!(
                "Card cannot be moved, because response column_id and request column_id is different"
            );
            return Err(TaskTrackerClientMoveToColumnError::MoveValidationFailed);
        }

        Ok(())
    }
}
