use crate::application::repository::commands::create_repository::command::CreateRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::Message;
use teloxide::prelude::*;
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminCreateRepositoryDispatcher {}

impl TelegramBotDialogueAdminCreateRepositoryDispatcher {
    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryName]
                    .endpoint(Self::handle_create_repository_name),
            )
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryOwner { name }]
                    .endpoint(Self::handle_create_repository_owner),
            )
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryFinish { name, owner }]
                    .endpoint(Self::handle_create_repository_finish),
            )
    }
}

impl TelegramBotDialogueAdminCreateRepositoryDispatcher {
    async fn handle_create_repository_name(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let name = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое название.")
                    .await?;
                return Ok(());
            }
        };

        dialogue
            .update(
                crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::CreateRepositoryOwner { name },
                ),
            )
            .await?;

        bot.send_message(msg.chat.id, "👤 Введите владельца репозитория (owner):")
            .await?;

        Ok(())
    }

    async fn handle_create_repository_owner(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let owner = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое значение.")
                    .await?;
                return Ok(());
            }
        };

        dialogue
            .update(
                crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::CreateRepositoryFinish { name, owner },
                ),
            )
            .await?;

        bot.send_message(msg.chat.id, "🔗 Введите URL репозитория:")
            .await?;

        Ok(())
    }

    async fn handle_create_repository_finish(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        executors: Arc<ApplicationBoostrapExecutors>,
        (name, owner): (String, String),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое значение.")
                    .await?;
                return Ok(());
            }
        };

        let cmd = CreateRepositoryCommand { name, owner, url };

        let loading_message = bot
            .send_message(msg.chat.id, "Создаем репозиторий...")
            .await?;

        match executors.commands.create_repository.execute(&cmd).await {
            Ok(response) => {
                bot.edit_message_text(
                    msg.chat.id,
                    loading_message.id,
                    format!(
                        "✅ Репозиторий <b>{}/{}</b> успешно создан (ID: {}).",
                        response.repository.owner,
                        response.repository.name,
                        response.repository.id.0
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create repository");
                bot.edit_message_text(
                    msg.chat.id,
                    loading_message.id,
                    format!("❌ Ошибка создания репозитория: {e}"),
                )
                .await?;
            }
        }

        dialogue.exit().await.ok();

        Ok(())
    }
}
