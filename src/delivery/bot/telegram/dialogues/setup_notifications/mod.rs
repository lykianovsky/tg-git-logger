use crate::application::repository::commands::set_repository_notifications_chat::command::SetRepositoryNotificationsChatCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use std::error::Error;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::{dptree, utils::html};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotSetupNotificationsState {
    #[default]
    SelectRepository,
}

pub struct TelegramBotSetupNotificationsDispatcher {}

impl TelegramBotSetupNotificationsDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query().branch(
            case![TelegramBotSetupNotificationsState::SelectRepository].endpoint(handle_select),
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
        Err(_) => return Ok(()),
    };

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    match executors
        .commands
        .set_repository_notifications_chat
        .execute(&SetRepositoryNotificationsChatCommand {
            repository_id: RepositoryId(repo_id),
            notifications_chat_id: SocialChatId(chat_id.0),
        })
        .await
    {
        Ok(r) => {
            let owner = html::escape(&r.repository.owner);
            let name = html::escape(&r.repository.name);
            bot.edit_message_text(
                chat_id,
                message_id,
                t!(
                    "telegram_bot.dialogues.setup_notifications.bound_success",
                    owner = owner,
                    name = name
                )
                .to_string(),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to set notifications chat");
            bot.edit_message_text(
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.setup_notifications.bind_error").to_string(),
            )
            .await?;
        }
    }

    dialogue.exit().await.ok();
    Ok(())
}
