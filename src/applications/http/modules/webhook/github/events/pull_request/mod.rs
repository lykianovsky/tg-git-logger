use crate::applications::state::ApplicationState;
use crate::domain::notification::use_cases::notify_received_webhook::NotifyReceivedWebhookUseCase;
use crate::domain::task::use_cases::move_to_test_board::MoveTaskToTestBoardUseCase;
use crate::domain::task_tracker::service::TaskTrackerTaskId;
use crate::infrastructure::contracts::github::payloads::pull_request::GithubPullRequestEvent;
use crate::unwrap_or_return_status;
use axum::http::StatusCode;
use serde_json::Value;
use std::sync::Arc;

pub struct GithubWebhookPullRequestHandler {
    state: Arc<ApplicationState>
}

impl GithubWebhookPullRequestHandler {
    pub fn new(state: Arc<ApplicationState>) -> Self {
        Self {
            state
        }
    }

    pub fn handle(&self, payload: Value) -> StatusCode {
        let event = unwrap_or_return_status!(
            serde_json::from_value::<GithubPullRequestEvent>(payload),
            StatusCode::BAD_REQUEST
        );
        
        let received_use_case = NotifyReceivedWebhookUseCase::new(
            Arc::clone(&self.state.services.notifier),
            Arc::clone(&self.state.services.task_tracker),
        );

        received_use_case.execute(&event.build());

        tracing::debug!("Parsed GitHub pull request event: {:?}", event);

        if event.pull_request.merged {
            self.on_merge(&event);
        }

        StatusCode::NO_CONTENT
    }

    fn on_merge(&self, event: &GithubPullRequestEvent) {
        tracing::debug!("Preparing MoveTaskToTestBoardUseCase for execution");

        let move_task_to_test = Arc::new(MoveTaskToTestBoardUseCase::new(
            Arc::clone(&self.state.services.notifier),
            Arc::clone(&self.state.services.task_tracker)
        ));

        let task_id = match self.state.services.task_tracker.extract_task_id(&event.pull_request.title) {
            Some(id) => id,
            None => {
                tracing::warn!("Task ID not found in pull request title: {}", event.pull_request.title);
                return
            }
        };

        tokio::spawn(async move {
            tracing::debug!("Executing MoveTaskToTestBoardUseCase for card ID: {:?}", task_id);

            let _ = move_task_to_test.execute(&task_id).await;
        });
    }
}
