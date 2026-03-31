mod create;
mod edit;

use crate::application::repository::commands::delete_repository::command::DeleteRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
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
use crate::utils::builder::message::MessageBuilder;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
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
                case![TelegramBotDialogueAdminState::ViewRepositorySelect]
                    .endpoint(Self::handle_view_select),
            )
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
            TelegramBotAdminRepositoryAction::View => {
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
                        t!("telegram_bot.dialogues.admin.repository.no_repositories_info")
                            .to_string(),
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
                            format!("repo_view_select_{}", r.id.0),
                        )]
                    })
                    .collect();

                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ViewRepositorySelect,
                    ))
                    .await?;

                bot.edit_message_text(
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.admin.repository.select_for_view").to_string(),
                )
                .reply_markup(InlineKeyboardMarkup::new(buttons))
                .await?;
            }
            TelegramBotAdminRepositoryAction::Create => {
                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::CreateRepositoryName,
                    ))
                    .await?;
                bot.edit_message_text(
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.admin.repository.enter_name").to_string(),
                )
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
                        t!("telegram_bot.dialogues.admin.repository.no_repos_for_edit").to_string(),
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
                    t!("telegram_bot.dialogues.admin.repository.select_for_edit").to_string(),
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
                    bot.edit_message_text(
                        chat_id,
                        message_id,
                        t!("telegram_bot.dialogues.admin.repository.no_repos_for_delete")
                            .to_string(),
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
                            format!("repo_delete_select_{}", r.id.0),
                        )]
                    })
                    .collect();

                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::DeleteRepositorySelect,
                    ))
                    .await?;

                bot.edit_message_text(
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.admin.repository.select_for_delete").to_string(),
                )
                    .reply_markup(InlineKeyboardMarkup::new(buttons))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_view_select(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let repository_id: i32 = match data
            .strip_prefix("repo_view_select_")
            .and_then(|s| s.parse().ok())
        {
            Some(id) => id,
            None => {
                tracing::error!(data = %data, "Invalid repo_view_select callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let repo = match executors
            .commands
            .create_repository
            .repository_repo
            .find_by_id(RepositoryId(repository_id))
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, repository_id, "Repository not found for view");
                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    t!("telegram_bot.dialogues.admin.repository.not_found").to_string(),
                )
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
                dialogue.exit().await.ok();
                return Ok(());
            }
        };

        let chat_info = match repo.social_chat_id {
            Some(id) => id.0.to_string(),
            None => t!("telegram_bot.dialogues.admin.repository.global_chat").to_string(),
        };

        let text = MessageBuilder::new()
            .bold(&format!("📦 {}/{}", repo.owner, repo.name))
            .section("URL", &repo.url)
            .section(
                t!("telegram_bot.dialogues.admin.repository.notifications").as_ref(),
                &chat_info,
            )
            .section(
                t!("telegram_bot.dialogues.admin.repository.created_at").as_ref(),
                &repo.created_at.format("%d %b %Y, %H:%M UTC").to_string(),
            )
            .section(
                t!("telegram_bot.dialogues.admin.repository.updated_at").as_ref(),
                &repo.updated_at.format("%d %b %Y, %H:%M UTC").to_string(),
            )
            .build();

        bot.edit_message_text(msg.chat().id, msg.id(), text)
            .parse_mode(ParseMode::Html)
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;

        dialogue.exit().await.ok();
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
            InlineKeyboardButton::callback(
                t!("telegram_bot.dialogues.admin.repository.confirm_delete").to_string(),
                "repo_delete_yes",
            ),
            InlineKeyboardButton::callback(
                t!("telegram_bot.common.cancel").to_string(),
                "repo_delete_cancel",
            ),
        ]]);

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::DeleteRepositoryConfirm { repository_id },
            ))
            .await?;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            t!("telegram_bot.dialogues.admin.repository.delete_confirm", name = repo_label)
                .to_string(),
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
            bot.edit_message_text(
                msg.chat().id,
                msg.id(),
                t!("telegram_bot.dialogues.admin.repository.delete_cancelled").to_string(),
            )
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
                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    t!("telegram_bot.dialogues.admin.repository.deleted").to_string(),
                )
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to delete repository");
                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    t!("telegram_bot.dialogues.admin.repository.delete_error").to_string(),
                )
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
            }
        }

        dialogue.exit().await.ok();
        Ok(())
    }
}
