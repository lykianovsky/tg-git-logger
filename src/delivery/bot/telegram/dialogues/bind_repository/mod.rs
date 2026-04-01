use crate::application::user::commands::bind_repository::command::BindRepositoryCommand;
use crate::application::user::commands::bind_repository::error::BindRepositoryExecutorError;
use crate::application::user::commands::unbind_repository::command::UnbindRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::confirm::TelegramBotConfirmAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
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
            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.bind_repository.bound_success").to_string(),
            )
            .await?;
            dialogue.exit().await.ok();
        }
        Err(BindRepositoryExecutorError::AlreadyBound) => {
            let keyboard = KeyboardBuilder::new()
                .row::<TelegramBotConfirmAction>(vec![
                    TelegramBotConfirmAction::Yes,
                    TelegramBotConfirmAction::No,
                ])
                .build();

            dialogue
                .update(TelegramBotDialogueState::BindRepository(
                    TelegramBotBindRepositoryState::ConfirmUnbind { repository_id },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.bind_repository.already_bound").to_string(),
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

    let action = match TelegramBotConfirmAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => {
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    if matches!(action, TelegramBotConfirmAction::No) {
        bot.send_message(msg.chat().id, t!("telegram_bot.common.cancelled").to_string()).await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    let cmd = UnbindRepositoryCommand {
        social_user_id: SocialUserId(query.from.id.0 as i32),
        repository_id: RepositoryId(repository_id),
    };

    match executors.commands.unbind_repository.execute(&cmd).await {
        Ok(_) => {
            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.bind_repository.unbound_success").to_string(),
            )
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
