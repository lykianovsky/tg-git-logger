use crate::application::repository::commands::create_repository::command::CreateRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::admin::helpers::extract_text;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminRepositoryCreateDispatcher {}

impl TelegramBotDialogueAdminRepositoryCreateDispatcher {
    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryName]
                    .endpoint(Self::handle_name),
            )
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryOwner { name }]
                    .endpoint(Self::handle_owner),
            )
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryFinish { name, owner }]
                    .endpoint(Self::handle_finish),
            )
    }

    async fn handle_name(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let name = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    t!("telegram_bot.dialogues.admin.repository.create.name_required").to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::CreateRepositoryOwner { name },
            ))
            .await?;

        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.admin.repository.create.enter_owner").to_string(),
        )
        .await?;
        Ok(())
    }

    async fn handle_owner(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let owner = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    t!("telegram_bot.dialogues.admin.repository.create.owner_required").to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::CreateRepositoryFinish { name, owner },
            ))
            .await?;

        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.admin.repository.create.enter_url").to_string(),
        )
        .await?;
        Ok(())
    }

    async fn handle_finish(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        executors: Arc<ApplicationBoostrapExecutors>,
        (name, owner): (String, String),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    t!("telegram_bot.dialogues.admin.repository.create.url_required").to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        let loading = bot
            .send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.admin.repository.create.loading").to_string(),
            )
            .await?;

        match executors
            .commands
            .create_repository
            .execute(&CreateRepositoryCommand { name, owner, url })
            .await
        {
            Ok(r) => {
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    t!(
                        "telegram_bot.dialogues.admin.repository.create.success",
                        owner = r.repository.owner,
                        name = r.repository.name
                    )
                    .to_string(),
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create repository");
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    t!("telegram_bot.dialogues.admin.repository.create.db_error").to_string(),
                )
                .await?;
            }
        }

        dialogue.exit().await.ok();
        Ok(())
    }
}
