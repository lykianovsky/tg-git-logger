use crate::config::environment::ENV;
use crate::domain::notification::service::{NotificationSendToChatError, NotificationService};
use crate::domain::task_tracker::service::{TaskTrackerColumnId, TaskTrackerMoveCardError, TaskTrackerService, TaskTrackerTaskId};
use crate::utils::builder::message::MessageBuilder;
use std::fmt::format;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoveTaskError {
    #[error("task tracker error: {0}")]
    TaskTracker(TaskTrackerMoveCardError),

    #[error("notification error: {0}")]
    Notification(NotificationSendToChatError),
}


pub struct MoveTaskToTestBoardUseCase {
    notifier: Arc<dyn NotificationService>,
    task_tracker: Arc<dyn TaskTrackerService>
}

impl MoveTaskToTestBoardUseCase {
    pub fn new(
        notifier: Arc<dyn NotificationService>,
        task_tracker: Arc<dyn TaskTrackerService>
    ) -> Self {
        Self {
            notifier,
            task_tracker
        }
    }

    pub async fn execute(&self, card_id: &TaskTrackerTaskId) -> Result<(), MoveTaskError> {
        let column_id = TaskTrackerColumnId(ENV.get("TASK_TRACKER_QA_COLUMN_ID"));
        let chat_id: i64 = ENV.get("TELEGRAM_CHAT_ID").parse().unwrap();

        if let Err(error) = self.task_tracker.move_card(card_id, &column_id).await {
            tracing::error!("Failed to move task to test board: {:?}", error);
            return Err(MoveTaskError::TaskTracker(error));
        }

        let log_message = MessageBuilder::new()
            .with_html_escape(true)
            .line(&format!(
                "Задача {} была перенесена в колонку для тестирования",
                self.task_tracker.create_task_link(card_id, &format!("#{}", card_id))
            ));

        if let Err(error) = self.notifier.send_to_chat(chat_id, &log_message).await {
            tracing::error!("Failed to send text: {:?}", error);
            return Err(MoveTaskError::Notification(error));
        }

        tracing::info!("Successfully moved task id: {} to test board.", &card_id.0);
        Ok(())
    }
}