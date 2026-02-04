use crate::client::kaiten::{KaitenClient, KaitenClientBase, KaitenClientToken};
use crate::config::environment::ENV;
use crate::domain::task_tracker::service::{TaskTrackerColumnId, TaskTrackerMoveCardError, TaskTrackerService, TaskTrackerTaskId};
use regex::Regex;

pub struct KaitenTaskTrackerService {
    client: KaitenClient
}

impl KaitenTaskTrackerService {
    pub fn new() -> KaitenTaskTrackerService {
        Self {
            client: KaitenClient::new(
                KaitenClientBase(
                    ENV.get("TASK_TRACKER_BASE")
                ),
                KaitenClientToken(
                    ENV.get("TASK_TRACKER_API_TOKEN")
                )
            )
        }
    }
}


#[async_trait::async_trait]
impl TaskTrackerService for KaitenTaskTrackerService {
    async fn move_card(&self, task_id: &TaskTrackerTaskId, column_id: &TaskTrackerColumnId) -> Result<(), TaskTrackerMoveCardError>
    {
        self.client.move_card(&task_id.0, &column_id.0).await.map_err(|err| {
            tracing::error!("Failed to move card: {:?}", err);
            TaskTrackerMoveCardError::RequestFailed
        })
    }

    fn linkify_tasks_in_text(&self, text: &str) -> String {
        let pattern = ENV.get("TASK_TRACKER_REGEXP");
        let regex = Regex::new(pattern.as_str()).unwrap();

        regex
            .replace_all(text, |caps: &regex::Captures| {
                let original_text = &caps[0];
                let id = &caps[1];
                let link = ENV.get("TASK_TRACKER_PATH_TO_CARD").replace("{id}", id);
                format!("<a href=\"{}\">{}</a>", link, original_text)
            })
            .to_string()
    }

    fn create_task_link(&self, id: &TaskTrackerTaskId, text: &str) -> String {
        let link = ENV.get("TASK_TRACKER_PATH_TO_CARD").replace("{id}", id.0.as_str());
        format!("<a href=\"{}\">{}</a>", link, text)
    }

    fn extract_task_id(&self, text: &str) -> Option<TaskTrackerTaskId> {
        let pattern = ENV.get("TASK_TRACKER_REGEXP");
        let regex = Regex::new(pattern.as_str()).ok()?;

        regex
            .captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| TaskTrackerTaskId(m.as_str().to_string()))
    }
}