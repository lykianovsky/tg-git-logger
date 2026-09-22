use crate::application::test_run::commands::ingest_test_run_result::command::{
    IngestTestRunResultCommand, KnownTestRunState,
};
use crate::application::test_run::commands::ingest_test_run_result::executor::IngestTestRunResultExecutor;
use crate::application::test_run::queries::get_test_run_progress::executor::GetTestRunProgressExecutor;
use crate::delivery::notifications::test_run::{
    build_test_run_card, deliver_test_run_update, resolve_test_run_chat_id,
};
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::test_run::value_objects::run_tag::RunTag;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::webhook::events::workflow::WebhookWorkflowEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use async_trait::async_trait;
use std::sync::Arc;

/// Итоги прогона тестов: прогон узнаём по метке в имени, которую бот передал в CI.
/// Чужие workflow до записи не доходят — метки в их имени нет.
pub struct WebhookTestRunResultListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub ingest_test_run_result: Arc<IngestTestRunResultExecutor>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub default_chat_id: SocialChatId,
    pub notification_service: Arc<dyn NotificationService>,
    pub get_test_run_progress: Arc<GetTestRunProgressExecutor>,
}

/// Время в вебхуке приходит строкой RFC 3339
fn parse_time(value: Option<&str>) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(value?)
        .ok()
        .map(|time| time.with_timezone(&chrono::Utc))
}

#[async_trait]
impl EventListener<WebhookWorkflowEvent> for WebhookTestRunResultListener {
    async fn handle(&self, payload: &WebhookWorkflowEvent) {
        let Some(run_tag) = RunTag::extract_from_run_name(&payload.name) else {
            return;
        };

        let response = match self
            .ingest_test_run_result
            .execute(&IngestTestRunResultCommand {
                run_tag: run_tag.clone(),
                known_state: Some(KnownTestRunState {
                    provider_run_id: payload.id,
                    status: payload.status.clone(),
                    conclusion: payload.conclusion.clone(),
                    run_url: payload.html_url.clone(),
                    sha: match payload.head_sha.is_empty() {
                        true => None,
                        false => Some(payload.head_sha.clone()),
                    },
                    started_at: parse_time(payload.created_at.as_deref()),
                    finished_at: match payload.status.as_str() {
                        "completed" => parse_time(payload.updated_at.as_deref()),
                        _ => None,
                    },
                }),
            })
            .await
        {
            Ok(response) => response,
            Err(error) => {
                tracing::warn!(%error, run_tag = %run_tag, "Failed to ingest test run result");

                return;
            }
        };

        // Прогон ещё идёт — сообщение шлём один раз, по итогам
        if response.run.is_active() {
            return;
        }

        let chat_id =
            resolve_test_run_chat_id(&self.repository_repo, &response.run, self.default_chat_id)
                .await;
        let (message, keyboard) = build_test_run_card(
            &self.repository_repo,
            &self.get_test_run_progress,
            &response.run,
        )
        .await;

        deliver_test_run_update(
            &self.notification_service,
            &self.publisher,
            &response.run,
            chat_id,
            message,
            Some(keyboard),
        )
        .await;
    }
}
