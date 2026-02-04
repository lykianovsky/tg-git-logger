mod entities;

use crate::client::kaiten::entities::KaitenCard;
use reqwest::{Client, Method, Response};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

pub struct KaitenClientBase(pub String);

pub struct KaitenClientToken(pub String);

#[derive(Error, Debug)]
pub enum KaitenClientError {
    #[error("Card move validation failed")]
    MoveValidationFailed,

    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),
}

pub struct KaitenClient {
    base: KaitenClientBase,
    token: KaitenClientToken,
    client: Client
}

impl KaitenClient {
    pub fn new(base: KaitenClientBase, token: KaitenClientToken) -> Self {
        Self {
            base,
            token,
            client: Client::new(),
        }
    }

    async fn request<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<Response, reqwest::Error> {
        let url = format!("{}/api/latest{}", self.base.0, path);

        tracing::info!("{}", url);

        let mut req = self
            .client
            .request(method, &url)
            .bearer_auth(&self.token.0)
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            req = req.json(body);
        }

        req.send().await
    }

    pub async fn move_card(&self, card_id: &str, column_id: &str) -> Result<(), KaitenClientError> {
        let body = json!({ "column_id": column_id });

        let resp = self
            .request(Method::PATCH, &format!("/cards/{}", card_id), Some(&body))
            .await?;

        let response = resp.json::<KaitenCard>().await?;

        let span = tracing::debug_span!("move_card", card_id = card_id, column_id = column_id, response_column_id = response.column_id);
        let _enter = span.enter();

        tracing::debug!("MoveCard response: {:?}", response);

        if response.column_id != column_id.parse::<u64>().unwrap() {
            tracing::error!("Card cannot be moved, because response column_id and request column_id is different");
            return Err(KaitenClientError::MoveValidationFailed)
        }

        Ok(())
    }
}