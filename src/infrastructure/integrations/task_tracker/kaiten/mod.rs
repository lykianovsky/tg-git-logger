use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::task::ports::task_tracker_client::{
    NewTaskTrackerCard, TaskTrackerBoard, TaskTrackerCard, TaskTrackerClient,
    TaskTrackerClientCreateCardError, TaskTrackerClientGetCardError, TaskTrackerClientListError,
    TaskTrackerClientMoveToColumnError, TaskTrackerColumn, TaskTrackerSpace, TaskTrackerTag,
    TaskTrackerUser,
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
pub struct KaitenUser {
    pub id: u64,
    pub full_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct KaitenTag {
    pub id: u64,
    pub name: String,
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

    async fn create_card(
        &self,
        card: &NewTaskTrackerCard,
    ) -> Result<TaskTrackerCard, TaskTrackerClientCreateCardError> {
        let body = json!({
            "board_id": card.board_id,
            "column_id": card.column_id,
            "title": card.title,
            "description": card.description,
            "responsible_id": card.responsible_id,
        });

        let created: KaitenCard = self
            .request(Method::POST, "/cards", Some(&body))
            .await
            .map_err(|error| TaskTrackerClientCreateCardError::ClientError(error.to_string()))?;

        // Тег вешается отдельным запросом: в создании карточки Kaiten его не принимает
        if let Some(tag) = card.tag.as_ref() {
            let tag_body = json!({ "name": tag });
            let attached: Result<serde_json::Value, _> = self
                .request(
                    Method::POST,
                    &format!("/cards/{}/tags", created.id),
                    Some(&tag_body),
                )
                .await;

            if let Err(error) = attached {
                tracing::warn!(error = %error, card_id = created.id, "Failed to attach card tag");
            }
        }

        Ok(TaskTrackerCard {
            id: TaskId(created.id),
            title: created.title,
            url: format!("{}/ticket/{}", self.base.0, created.id),
        })
    }

    async fn list_users(&self) -> Result<Vec<TaskTrackerUser>, TaskTrackerClientListError> {
        let users: Vec<KaitenUser> = self
            .request::<(), _>(Method::GET, "/users", None)
            .await
            .map_err(|error| TaskTrackerClientListError::ClientError(error.to_string()))?;

        Ok(users
            .into_iter()
            .map(|user| TaskTrackerUser {
                id: user.id,
                name: user
                    .full_name
                    .or(user.username)
                    .unwrap_or_else(|| user.id.to_string()),
            })
            .collect())
    }

    async fn list_tags(&self) -> Result<Vec<TaskTrackerTag>, TaskTrackerClientListError> {
        let tags: Vec<KaitenTag> = self
            .request::<(), _>(Method::GET, "/tags", None)
            .await
            .map_err(|error| TaskTrackerClientListError::ClientError(error.to_string()))?;

        Ok(tags
            .into_iter()
            .map(|tag| TaskTrackerTag {
                id: tag.id,
                name: tag.name,
            })
            .collect())
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
