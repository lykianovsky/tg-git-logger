use crate::applications::state::ApplicationState;
use crate::domain::notification::use_cases::notify_received_webhook::NotifyReceivedWebhookUseCase;
use crate::infrastructure::contracts::github::payloads::workflow::GithubWorkflowEvent;
use crate::unwrap_or_return_status;
use axum::http::StatusCode;
use serde_json::Value;
use std::sync::Arc;

pub struct GithubWebhookWorkflowHandler {
    state: Arc<ApplicationState>
}

impl GithubWebhookWorkflowHandler {
    pub fn new(state: Arc<ApplicationState>) -> Self {
        Self {
            state
        }
    }

    pub fn handle(&self, payload: Value) -> StatusCode {
        let event = unwrap_or_return_status!(serde_json::from_value::<GithubWorkflowEvent>(payload), StatusCode::BAD_REQUEST);;

        let received_use_case = NotifyReceivedWebhookUseCase::new(
            Arc::clone(&self.state.services.notifier),
            Arc::clone(&self.state.services.task_tracker),
        );

        received_use_case.execute(&event.build());

        StatusCode::NO_CONTENT
    }
}