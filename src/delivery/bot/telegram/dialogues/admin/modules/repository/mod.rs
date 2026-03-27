mod create;
mod edit;

use crate::application::repository::commands::delete_repository::command::DeleteRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::admin::helpers::db_error_message;
use crate::delivery::bot::telegram::dialogues::admin::modules::repository::create::TelegramBotDialogueAdminRepositoryCreateDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::repository::edit::TelegramBotDialogueAdminRepositoryEditDispatcher;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::admin_repository::TelegramBotAdminRepositoryAction;
use crate::domain::repository::entities::repository::Repository;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminRepositoryDispatcher {}

impl TelegramBotDialogueAdminRepositoryDispatcher {
    pub fn query_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureRepository]
                    .endpoint(Self::handle_menu),
            )
            .branch(TelegramBotDialogueAdminRepositoryEditDispatcher::query_branches())
            .branch(
                case![TelegramBotDialogueAdminState::DeleteRepositorySelect]
                    .endpoint(Self::handle_delete_select),
            )
            .branch(
                case![TelegramBotDialogueAdminState::DeleteRepositoryConfirm { repository_id }]
                    .endpoint(Self::handle_delete_confirm),
            )
    }

    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(TelegramBotDialogueAdminRepositoryCreateDispatcher::message_branches())
            .branch(TelegramBotDialogueAdminRepositoryEditDispatcher::message_branches())
    }
}

impl TelegramBotDialogueAdminRepositoryDispatcher {
    async fn handle_menu(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");

        let action = match TelegramBotAdminRepositoryAction::from_callback_data(data) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!(error = %e, "Unknown repository action");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let chat_id = msg.chat().id;
        let message_id = msg.id();

        match action {
            TelegramBotAdminRepositoryAction::Create => {
                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::CreateRepositoryName,
                    ))
                    .await?;
                bot.edit_message_text(chat_id, message_id, "📝 Введите название репозитория:")
                    .reply_markup(InlineKeyboardMarkup::default())
                    .await?;
            }
            TelegramBotAdminRepositoryAction::Edit => {
                let repositories: Vec<Repository> = executors
                    .commands
                    .create_repository
                    .repository_repo
                    .find_all()
                    .await
                    .unwrap_or_default();

                if repositories.is_empty() {
                    bot.edit_message_text(
                        chat_id,
                        message_id,
                        "ℹ️ Нет репозиториев для редактирования. Сначала создайте репозиторий.",
                    )
                    .await?;
                    dialogue.exit().await.ok();
                    return Ok(());
                }

                let buttons: Vec<Vec<InlineKeyboardButton>> = repositories
                    .into_iter()
                    .map(|r| {
                        vec![InlineKeyboardButton::callback(
                            format!("{}/{}", r.owner, r.name),
                            format!("repo_select_{}", r.id.0),
                        )]
                    })
                    .collect();

                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::EditRepositorySelect,
                    ))
                    .await?;

                bot.edit_message_text(
                    chat_id,
                    message_id,
                    "✏️ Выберите репозиторий для редактирования:",
                )
                .reply_markup(InlineKeyboardMarkup::new(buttons))
                .await?;
            }
            TelegramBotAdminRepositoryAction::Delete => {
                let repositories: Vec<Repository> = executors
                    .commands
                    .create_repository
                    .repository_repo
                    .find_all()
                    .await
                    .unwrap_or_default();

                if repositories.is_empty() {
                    bot.edit_message_text(chat_id, message_id, "ℹ️ Нет репозиториев для удаления.")
                        .await?;
                    dialogue.exit().await.ok();
                    return Ok(());
                }

                let buttons: Vec<Vec<InlineKeyboardButton>> = repositories
                    .into_iter()
                    .map(|r| {
                        vec![InlineKeyboardButton::callback(
                            format!("{}/{}", r.owner, r.name),
                            format!("repo_delete_select_{}", r.id.0),
                        )]
                    })
                    .collect();

                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::DeleteRepositorySelect,
                    ))
                    .await?;

                bot.edit_message_text(chat_id, message_id, "🗑 Выберите репозиторий для удаления:")
                    .reply_markup(InlineKeyboardMarkup::new(buttons))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_delete_select(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let repository_id: i32 = match data
            .strip_prefix("repo_delete_select_")
            .and_then(|s| s.parse().ok())
        {
            Some(id) => id,
            None => {
                tracing::error!(data = %data, "Invalid repo_delete_select callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let repo_label = executors
            .commands
            .create_repository
            .repository_repo
            .find_by_id(RepositoryId(repository_id))
            .await
            .map(|r| format!("{}/{}", r.owner, r.name))
            .unwrap_or_else(|_| format!("ID {}", repository_id));

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("✅ Да, удалить", "repo_delete_yes"),
            InlineKeyboardButton::callback("❌ Отмена", "repo_delete_cancel"),
        ]]);

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::DeleteRepositoryConfirm { repository_id },
            ))
            .await?;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            format!(
                "🗑 Удалить репозиторий <b>{}</b>?\n\nЭто действие необратимо.",
                repo_label
            ),
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

        Ok(())
    }

    async fn handle_delete_confirm(
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

        if data == "repo_delete_cancel" {
            bot.edit_message_text(msg.chat().id, msg.id(), "❌ Удаление отменено.")
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }

        if data != "repo_delete_yes" {
            return Ok(());
        }

        let cmd = DeleteRepositoryCommand {
            id: RepositoryId(repository_id),
        };
        match executors.commands.delete_repository.execute(&cmd).await {
            Ok(_) => {
                bot.edit_message_text(msg.chat().id, msg.id(), "✅ Репозиторий успешно удалён.")
                    .reply_markup(InlineKeyboardMarkup::default())
                    .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to delete repository");
                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    db_error_message("удалить репозиторий"),
                )
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
            }
        }

        dialogue.exit().await.ok();
        Ok(())
    }
}
