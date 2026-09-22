use crate::application::test_run::queries::build_test_report::executor::BuildTestReportExecutor;
use crate::application::test_run::queries::build_test_report::query::BuildTestReportQuery;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_message_id::SocialMessageId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use rust_i18n::t;
use std::sync::Arc;

/// Итог прогона уходит одинаково и из вебхука, и из синхронизации по расписанию —
/// поэтому текст и выбор чата живут в одном месте
pub fn build_test_run_message(run: &TestRun, report_url: Option<&str>) -> MessageBuilder {
    let totals = run.totals.unwrap_or_default();
    let status_key = format!("report.test_run.status.{}", run.status.as_str());

    let mut builder = MessageBuilder::new()
        .bold(&t!(&status_key).to_string())
        .empty_line()
        .with_html_escape(true)
        .section_code(
            &t!("telegram_bot.test_run.branch").to_string(),
            &run.git_ref,
        );

    if let Some(args) = run.args.as_deref() {
        builder = builder.section_code(&t!("telegram_bot.dialogues.tests.scope").to_string(), args);
    }

    builder = builder.section(
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
        builder = builder
            .empty_line()
            .link(&t!("telegram_bot.test_run.report").to_string(), url);
    }

    builder
}

/// Запуск из чата отвечает в тот же чат, ночной прогон — в чат репозитория
pub async fn resolve_test_run_chat_id(
    repository_repo: &Arc<dyn RepositoryRepository>,
    run: &TestRun,
    default_chat_id: SocialChatId,
) -> SocialChatId {
    if let Some(chat_id) = run.chat_id {
        return chat_id;
    }

    repository_repo
        .find_by_id(run.repository_id)
        .await
        .ok()
        .and_then(|repository| {
            repository
                .notifications_chat_id
                .or(repository.social_chat_id)
        })
        .unwrap_or(default_chat_id)
}

/// Пока прогон идёт, бот переписывает свою карточку: жать «Обновить» не нужно.
/// Если карточки нет (ночной прогон), отправляем обычное сообщение
pub async fn deliver_test_run_update(
    notification_service: &Arc<dyn NotificationService>,
    publisher: &Arc<dyn MessageBrokerPublisher>,
    run: &TestRun,
    chat_id: SocialChatId,
    message: MessageBuilder,
) {
    if let Some(message_id) = run.message_id {
        let edited = notification_service
            .edit_message(
                &SocialType::Telegram,
                &chat_id,
                &SocialMessageId(message_id),
                &message,
            )
            .await;

        match edited {
            Ok(()) => return,
            Err(error) => {
                tracing::warn!(%error, "Failed to update test run card, sending new message")
            }
        }
    }

    publisher
        .publish(&SendSocialNotifyJob {
            social_type: SocialType::Telegram,
            chat_id,
            message,
        })
        .await
        .ok();
}

pub async fn build_test_run_report_url(
    build_test_report: &Arc<BuildTestReportExecutor>,
    run: &TestRun,
) -> Option<String> {
    build_test_report
        .execute(&BuildTestReportQuery {
            test_run_id: run.id,
        })
        .await
        .map(|response| response.report_url)
        .inspect_err(|error| tracing::error!(%error, "Failed to build test report"))
        .ok()
}
