use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::task::ports::task_tracker_client::{
    TaskTrackerBoard, TaskTrackerCard, TaskTrackerClient, TaskTrackerClientGetCardError,
    TaskTrackerClientListError, TaskTrackerClientMoveToColumnError, TaskTrackerColumn,
    TaskTrackerSpace,
};
use crate::domain::task::value_objects::task_id::TaskId;
use reqwest::{Client, Method};
use serde::Serialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct KaitenCard {
    pub id: u64,
    pub title: String,
    pub column_id: u64,
}

#[derive(Deserialize, Debug)]
pub struct KaitenSpace {
    pub id: i32,
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct KaitenBoard {
    pub id: i32,
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct KaitenColumn {
    pub id: i32,
    pub title: String,
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

        tracing::debug!(method = %method, path = %path, "Kaiten API request");

        let mut req = self
            .client
            .request(method, &url)
            .bearer_auth(&self.token.0)
            .header("Content-Type", "application/json");

        if let Some(body) = body {
            req = req.json(body);
        }

        let resp = req.send().await?;

        let status = resp.status();
        let text = resp.text().await?;

        tracing::debug!(status = %status, path = %path, body_len = text.len(), "Kaiten API response");

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("Not found: {}", path).into());
        }

        let parsed = serde_json::from_str::<Response>(&text)?;

        Ok(parsed)
    }
}

#[async_trait]
impl TaskTrackerClient for KaitenClient {
    async fn move_task_to_column(
        &self,
        task_id: TaskId,
        column_id: u64,
    ) -> Result<(), TaskTrackerClientMoveToColumnError> {
        let body = json!({ "column_id": column_id });

        let response: KaitenCard = self
            .request(Method::PATCH, &format!("/cards/{}", task_id.0), Some(&body))
            .await
            .map_err(|e| TaskTrackerClientMoveToColumnError::ClientError(e.to_string()))?;

        let span = tracing::debug_span!(
            "move_card",
            task_id = task_id.0,
            column_id = column_id,
            response_column_id = response.column_id
        );
        let _enter = span.enter();

        if response.column_id != column_id {
            tracing::error!(
                "Card cannot be moved, because response column_id and request column_id is different"
            );
            return Err(TaskTrackerClientMoveToColumnError::MoveValidationFailed);
        }

        Ok(())
    }

    async fn get_card(
        &self,
        task_id: TaskId,
    ) -> Result<TaskTrackerCard, TaskTrackerClientGetCardError> {
        let card: KaitenCard = self
            .request::<(), KaitenCard>(Method::GET, &format!("/cards/{}", task_id.0), None)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("Not found") {
                    TaskTrackerClientGetCardError::NotFound
                } else {
                    TaskTrackerClientGetCardError::ClientError(msg)
                }
            })?;

        let url = format!("{}/space/0/boards/card/{}", self.base.0, card.id);

        Ok(TaskTrackerCard {
            id: TaskId(card.id),
            title: card.title,
            url,
        })
    }

    async fn list_spaces(&self) -> Result<Vec<TaskTrackerSpace>, TaskTrackerClientListError> {
        let spaces: Vec<KaitenSpace> = self
            .request::<(), Vec<KaitenSpace>>(Method::GET, "/spaces", None)
            .await
            .map_err(|e| TaskTrackerClientListError::ClientError(e.to_string()))?;

        Ok(spaces
            .into_iter()
            .map(|s| TaskTrackerSpace {
                id: s.id,
                title: s.title,
            })
            .collect())
    }

    async fn list_boards(
        &self,
        space_id: i32,
    ) -> Result<Vec<TaskTrackerBoard>, TaskTrackerClientListError> {
        let boards: Vec<KaitenBoard> = self
            .request::<(), Vec<KaitenBoard>>(
                Method::GET,
                &format!("/spaces/{}/boards", space_id),
                None,
            )
            .await
            .map_err(|e| TaskTrackerClientListError::ClientError(e.to_string()))?;

        Ok(boards
            .into_iter()
            .map(|b| TaskTrackerBoard {
                id: b.id,
                title: b.title,
            })
            .collect())
    }

    async fn list_columns(
        &self,
        board_id: i32,
    ) -> Result<Vec<TaskTrackerColumn>, TaskTrackerClientListError> {
        let columns: Vec<KaitenColumn> = self
            .request::<(), Vec<KaitenColumn>>(
                Method::GET,
                &format!("/boards/{}/columns", board_id),
                None,
            )
            .await
            .map_err(|e| TaskTrackerClientListError::ClientError(e.to_string()))?;

        Ok(columns
            .into_iter()
            .map(|c| TaskTrackerColumn {
                id: c.id,
                title: c.title,
            })
            .collect())
    }
}
