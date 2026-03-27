use crate::application::repository::commands::set_repository_notification_chat::command::SetRepositoryNotificationChatCommand;
use crate::application::repository::commands::unset_repository_notification_chat::command::UnsetRepositoryNotificationChatCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use teloxide::{Bot, dptree};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotSetupWebhookState {
    #[default]
    SelectRepository,
    ConfirmUnbind {
        repository_id: i32,
    },
    ConfirmRebind {
        repository_id: i32,
    },
}

pub struct TelegramBotSetupWebhookDispatcher {}

impl TelegramBotSetupWebhookDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query()
            .branch(case![TelegramBotSetupWebhookState::SelectRepository].endpoint(handle_select))
            .branch(
                case![TelegramBotSetupWebhookState::ConfirmUnbind { repository_id }]
                    .endpoint(handle_confirm_unbind),
            )
            .branch(
                case![TelegramBotSetupWebhookState::ConfirmRebind { repository_id }]
                    .endpoint(handle_confirm_rebind),
            );

        dptree::entry().branch(queries)
    }
}

async fn handle_select(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");
    let repo_id: i32 = match data.parse() {
        Ok(v) => v,
        Err(_) => {
            tracing::error!(data = %data, "Invalid repository_id in setup_webhook callback");
            return Ok(());
        }
    };

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let chat_id = msg.chat().id;

    let repository = match executors
        .commands
        .set_repository_notification_chat
        .repository_repo
        .find_by_id(RepositoryId(repo_id))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to find repository");
            bot.edit_message_text(chat_id, msg.id(), "❌ Репозиторий не найден.")
                .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    match repository.social_chat_id {
        Some(bound_chat) if bound_chat == SocialChatId(chat_id.0) => {
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("✅ Да, отвязать", "confirm"),
                InlineKeyboardButton::callback("❌ Нет", "cancel"),
            ]]);

            dialogue
                .update(TelegramBotDialogueState::SetupWebhook(
                    TelegramBotSetupWebhookState::ConfirmUnbind {
                        repository_id: repo_id,
                    },
                ))
                .await?;

            bot.edit_message_text(
                chat_id,
                msg.id(),
                format!(
                    "Этот чат уже привязан к репозиторию <b>{}/{}</b>.\nОтвязать?",
                    teloxide::utils::html::escape(&repository.owner),
                    teloxide::utils::html::escape(&repository.name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
        }
        Some(_) => {
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("✅ Да, перепривязать", "confirm"),
                InlineKeyboardButton::callback("❌ Нет", "cancel"),
            ]]);

            dialogue
                .update(TelegramBotDialogueState::SetupWebhook(
                    TelegramBotSetupWebhookState::ConfirmRebind {
                        repository_id: repo_id,
                    },
                ))
                .await?;

            bot.edit_message_text(
                chat_id,
                msg.id(),
                format!(
                    "Репозиторий <b>{}/{}</b> уже привязан к другому чату.\n\
                     Перепривязать к этому чату?",
                    teloxide::utils::html::escape(&repository.owner),
                    teloxide::utils::html::escape(&repository.name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
        }
        None => {
            bind_chat(&bot, &dialogue, &executors, repo_id, chat_id, msg.id()).await?;
        }
    }

    Ok(())
}

async fn handle_confirm_unbind(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    repository_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    if query.data.as_deref() == Some("cancel") {
        bot.edit_message_text(msg.chat().id, msg.id(), "Отменено.")
            .await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    match executors
        .commands
        .unset_repository_notification_chat
        .execute(&UnsetRepositoryNotificationChatCommand {
            repository_id: RepositoryId(repository_id),
        })
        .await
    {
        Ok(r) => {
            bot.edit_message_text(
                msg.chat().id,
                msg.id(),
                format!(
                    "✅ Чат отвязан от репозитория <b>{}/{}</b>.\n\
                     Уведомления будут приходить в чат по умолчанию.",
                    teloxide::utils::html::escape(&r.repository.owner),
                    teloxide::utils::html::escape(&r.repository.name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to unset notification chat");
            bot.edit_message_text(
                msg.chat().id,
                msg.id(),
                "❌ Не удалось отвязать чат. Попробуйте позже.",
            )
            .await?;
        }
    }

    dialogue.exit().await.ok();
    Ok(())
}

async fn handle_confirm_rebind(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    repository_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    if query.data.as_deref() == Some("cancel") {
        bot.edit_message_text(msg.chat().id, msg.id(), "Отменено.")
            .await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    bind_chat(
        &bot,
        &dialogue,
        &executors,
        repository_id,
        msg.chat().id,
        msg.id(),
    )
    .await
}

async fn bind_chat(
    bot: &Bot,
    dialogue: &TelegramBotDialogueType,
    executors: &Arc<ApplicationBoostrapExecutors>,
    repository_id: i32,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match executors
        .commands
        .set_repository_notification_chat
        .execute(&SetRepositoryNotificationChatCommand {
            repository_id: RepositoryId(repository_id),
            social_chat_id: SocialChatId(chat_id.0),
        })
        .await
    {
        Ok(r) => {
            bot.edit_message_text(
                chat_id,
                message_id,
                format!(
                    "✅ Чат привязан к репозиторию <b>{}/{}</b>.\n\
                     Вебхук-уведомления теперь будут приходить сюда.",
                    teloxide::utils::html::escape(&r.repository.owner),
                    teloxide::utils::html::escape(&r.repository.name),
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to set notification chat");
            bot.edit_message_text(
                chat_id,
                message_id,
                "❌ Не удалось привязать чат. Попробуйте позже.",
            )
            .await?;
        }
    }

    dialogue.exit().await.ok();
    Ok(())
}
