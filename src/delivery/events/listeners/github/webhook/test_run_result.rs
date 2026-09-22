use crate::application::test_run::commands::ingest_test_run_result::command::IngestTestRunResultCommand;
use crate::application::test_run::commands::ingest_test_run_result::executor::IngestTestRunResultExecutor;
use crate::application::test_run::queries::build_test_report::executor::BuildTestReportExecutor;
use crate::application::test_run::queries::build_test_report::query::BuildTestReportQuery;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::test_run::value_objects::run_tag::RunTag;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::workflow::WebhookWorkflowEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use rust_i18n::t;
use std::sync::Arc;

/// Итоги прогона тестов: прогон узнаём по метке в имени, которую бот передал в CI.
/// Чужие workflow до записи не доходят — метки в их имени нет.
pub struct WebhookTestRunResultListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub ingest_test_run_result: Arc<IngestTestRunResultExecutor>,
    pub build_test_report: Arc<BuildTestReportExecutor>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub default_chat_id: SocialChatId,
}

impl WebhookTestRunResultListener {
    /// Запуск из чата отвечает в тот же чат, ночной прогон — в чат репозитория
    async fn resolve_chat_id(&self, run: &TestRun) -> SocialChatId {
        if let Some(chat_id) = run.chat_id {
            return chat_id;
        }

        let repository = self
            .repository_repo
            .find_by_id(run.repository_id)
            .await
            .ok();

        repository
            .and_then(|repository| {
                repository
                    .notifications_chat_id
                    .or(repository.social_chat_id)
            })
            .unwrap_or(self.default_chat_id)
    }

    async fn build_report_url(&self, run: &TestRun) -> Option<String> {
        self.build_test_report
            .execute(&BuildTestReportQuery {
                test_run_id: run.id,
            })
            .await
            .map(|response| response.report_url)
            .inspect_err(|error| tracing::error!(%error, "Failed to build test report"))
            .ok()
    }

    fn build_message(run: &TestRun, report_url: Option<&str>) -> MessageBuilder {
        let totals = run.totals.unwrap_or_default();
        let status_key = format!("report.test_run.status.{}", run.status.as_str());

        let mut builder = MessageBuilder::new()
            .bold(&t!(&status_key).to_string())
            .empty_line()
            .with_html_escape(true)
            .section_code(
                &t!("telegram_bot.test_run.branch").to_string(),
                &run.git_ref,
            )
            .section(
                &t!("telegram_bot.test_run.totals").to_string(),
                &t!(
                    "telegram_bot.test_run.totals_value",
                    passed = totals.passed,
                    failed = totals.failed,
                    flaky = totals.flaky,
                    skipped = totals.skipped
                )
                .to_string(),
            );

        if let Some(url) = report_url {
            builder = builder.empty_line().section(
                &t!("telegram_bot.test_run.report").to_string(),
                &format!(
                    "<a href=\"{}\">{}</a>",
                    MessageBuilder::escape_html(url),
                    t!("telegram_bot.test_run.report_link")
                ),
            );
        }

        builder
    }
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

        let chat_id = self.resolve_chat_id(&response.run).await;
        let report_url = self.build_report_url(&response.run).await;

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id,
                message: Self::build_message(&response.run, report_url.as_deref()),
            })
            .await
            .ok();
    }
}
