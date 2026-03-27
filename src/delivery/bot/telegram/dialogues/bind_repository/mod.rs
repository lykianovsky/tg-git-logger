use crate::application::user::commands::bind_repository::command::BindRepositoryCommand;
use crate::application::user::commands::bind_repository::error::BindRepositoryExecutorError;
use crate::application::user::commands::unbind_repository::command::UnbindRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{Bot, dptree};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotBindRepositoryState {
    #[default]
    SelectRepository,
    ConfirmUnbind {
        repository_id: i32,
    },
}

pub struct TelegramBotBindRepositoryDispatcher {}

impl TelegramBotBindRepositoryDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query()
            .branch(case![TelegramBotBindRepositoryState::SelectRepository].endpoint(handle_select))
            .branch(
                case![TelegramBotBindRepositoryState::ConfirmUnbind { repository_id }]
                    .endpoint(handle_confirm_unbind),
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
    let repository_id: i32 = match data.parse() {
        Ok(v) => v,
        Err(_) => {
            tracing::error!(data = %data, "Invalid repository_id in bind_repository callback");
            return Ok(());
        }
    };

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);
    let rid = RepositoryId(repository_id);

    let cmd = BindRepositoryCommand {
        social_user_id,
        repository_id: rid,
    };

    match executors.commands.bind_repository.execute(&cmd).await {
        Ok(_) => {
            bot.send_message(msg.chat().id, "✅ Вы успешно привязались к репозиторию!")
                .await?;
            dialogue.exit().await.ok();
        }
        Err(BindRepositoryExecutorError::AlreadyBound) => {
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("✅ Да, отвязаться", repository_id.to_string()),
                InlineKeyboardButton::callback("❌ Нет", "cancel"),
            ]]);

            dialogue
                .update(TelegramBotDialogueState::BindRepository(
                    TelegramBotBindRepositoryState::ConfirmUnbind { repository_id },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                "Вы уже привязаны к этому репозиторию. Отвязаться?",
            )
            .reply_markup(keyboard)
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to bind repository");
            bot.send_message(msg.chat().id, format!("❌ Ошибка: {e}"))
                .await?;
            dialogue.exit().await.ok();
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

    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    if data == "cancel" {
        bot.send_message(msg.chat().id, "Отменено.").await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    let cmd = UnbindRepositoryCommand {
        social_user_id: SocialUserId(query.from.id.0 as i32),
        repository_id: RepositoryId(repository_id),
    };

    match executors.commands.unbind_repository.execute(&cmd).await {
        Ok(_) => {
            bot.send_message(msg.chat().id, "✅ Вы успешно отвязались от репозитория.")
                .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to unbind repository");
            bot.send_message(msg.chat().id, format!("❌ Ошибка: {e}"))
                .await?;
        }
    }

    dialogue.exit().await.ok();
    Ok(())
}
