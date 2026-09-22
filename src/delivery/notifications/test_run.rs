use crate::application::test_run::queries::get_test_run_progress::executor::GetTestRunProgressExecutor;
use crate::application::test_run::queries::get_test_run_progress::query::GetTestRunProgressQuery;
use crate::delivery::bot::telegram::dialogues::tests::card::{
    build_card_notification_keyboard, build_card_text,
};
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::notification::services::notification_service::{
    NotificationKeyboard, NotificationService,
};
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_message_id::SocialMessageId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;

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

/// Карточка прогона для автообновления: тот же вид, что и в чате
pub async fn build_test_run_card(
    repository_repo: &Arc<dyn RepositoryRepository>,
    get_test_run_progress: &Arc<GetTestRunProgressExecutor>,
    run: &TestRun,
) -> (MessageBuilder, NotificationKeyboard) {
    let repository_title = repository_repo
        .find_by_id(run.repository_id)
        .await
        .map(|repository| format!("{}/{}", repository.owner, repository.name))
        .unwrap_or_default();

    // Прогресс есть только у идущего прогона, и он стоит запроса в CI
    let progress = match run.is_active() {
        true => get_test_run_progress
            .execute(&GetTestRunProgressQuery { run: run.clone() })
            .await
            .ok()
            .and_then(|response| response.progress),
        false => None,
    };

    let text = build_card_text(Some(run), true, &repository_title, progress.as_ref());
    let keyboard = build_card_notification_keyboard(Some(run), true);

    (MessageBuilder::new().raw(&text), keyboard)
}

/// Пока прогон идёт, бот переписывает свою карточку: жать «Обновить» не нужно.
/// Если карточки нет (ночной прогон), отправляем обычное сообщение
pub async fn deliver_test_run_update(
    notification_service: &Arc<dyn NotificationService>,
    publisher: &Arc<dyn MessageBrokerPublisher>,
    run: &TestRun,
    chat_id: SocialChatId,
    message: MessageBuilder,
    keyboard: Option<NotificationKeyboard>,
) {
    if let Some(message_id) = run.message_id {
        let edited = notification_service
            .edit_message(
                &SocialType::Telegram,
                &chat_id,
                &SocialMessageId(message_id),
                &message,
                keyboard.as_ref(),
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
