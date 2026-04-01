use crate::application::repository::commands::update_repository::command::UpdateRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::admin::helpers::extract_text;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::admin_repository::REPO_SELECT_PREFIX;
use crate::delivery::bot::telegram::keyboards::actions::admin_repository_edit_field::TelegramBotAdminRepositoryEditField;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::{Bot, dptree};

enum EditField {
    Name,
    Owner,
    Url,
}

pub struct TelegramBotDialogueAdminRepositoryEditDispatcher {}

impl TelegramBotDialogueAdminRepositoryEditDispatcher {
    pub fn query_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::EditRepositorySelect]
                    .endpoint(Self::handle_select),
            )
            .branch(
                case![TelegramBotDialogueAdminState::EditRepositoryMenu { repository_id }]
                    .endpoint(Self::handle_field_choice),
            )
    }

    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::EditRepositoryName { repository_id }]
                    .endpoint(Self::handle_edit_name),
            )
            .branch(
                case![TelegramBotDialogueAdminState::EditRepositoryOwner { repository_id }]
                    .endpoint(Self::handle_edit_owner),
            )
            .branch(
                case![TelegramBotDialogueAdminState::EditRepositoryUrl { repository_id }]
                    .endpoint(Self::handle_edit_url),
            )
    }

    async fn handle_select(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let repository_id: i32 = match data
            .strip_prefix(REPO_SELECT_PREFIX)
            .and_then(|s| s.parse().ok())
        {
            Some(id) => id,
            None => {
                tracing::error!(data = %data, "Invalid repo_select callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let current_info = match executors
            .commands
            .create_repository
            .repository_repo
            .find_by_id(RepositoryId(repository_id))
            .await
        {
            Ok(r) => format!("📦 <b>{}/{}</b>\n🔗 {}", r.owner, r.name, r.url),
            Err(_) => {
                t!("telegram_bot.dialogues.admin.repository.data_unavailable").to_string()
            }
        };

        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotAdminRepositoryEditField>(vec![
                TelegramBotAdminRepositoryEditField::Name,
                TelegramBotAdminRepositoryEditField::Owner,
            ])
            .row::<TelegramBotAdminRepositoryEditField>(vec![
                TelegramBotAdminRepositoryEditField::Url,
            ])
            .build();

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::EditRepositoryMenu { repository_id },
            ))
            .await?;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            format!(
            "{}{}",
            current_info,
            t!("telegram_bot.dialogues.admin.repository.what_to_edit")
        ),
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

        Ok(())
    }

    async fn handle_field_choice(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        query: CallbackQuery,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let field = match TelegramBotAdminRepositoryEditField::from_callback_data(data) {
            Ok(f) => f,
            Err(e) => {
                tracing::error!(error = %e, "Unknown field");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let (next_state, prompt) = match field {
            TelegramBotAdminRepositoryEditField::Name => (
                TelegramBotDialogueAdminState::EditRepositoryName { repository_id },
                t!("telegram_bot.dialogues.admin.repository.edit.enter_name"),
            ),
            TelegramBotAdminRepositoryEditField::Owner => (
                TelegramBotDialogueAdminState::EditRepositoryOwner { repository_id },
                t!("telegram_bot.dialogues.admin.repository.edit.enter_owner"),
            ),
            TelegramBotAdminRepositoryEditField::Url => (
                TelegramBotDialogueAdminState::EditRepositoryUrl { repository_id },
                t!("telegram_bot.dialogues.admin.repository.edit.enter_url"),
            ),
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(next_state))
            .await?;

        bot.edit_message_text(msg.chat().id, msg.id(), prompt.to_string())
            .reply_markup(teloxide::types::InlineKeyboardMarkup::default())
            .await?;
        Ok(())
    }

    async fn handle_edit_name(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let new_value = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    t!("telegram_bot.dialogues.admin.repository.edit.name_required").to_string(),
                )
                .await?;
                return Ok(());
            }
        };
        Self::apply_field_edit(
            bot,
            dialogue,
            executors,
            msg,
            repository_id,
            EditField::Name,
            new_value,
        )
        .await
    }

    async fn handle_edit_owner(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let new_value = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    t!("telegram_bot.dialogues.admin.repository.edit.owner_required").to_string(),
                )
                .await?;
                return Ok(());
            }
        };
        Self::apply_field_edit(
            bot,
            dialogue,
            executors,
            msg,
            repository_id,
            EditField::Owner,
            new_value,
        )
        .await
    }

    async fn handle_edit_url(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let new_value = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    t!("telegram_bot.dialogues.admin.repository.edit.url_required").to_string(),
                )
                .await?;
                return Ok(());
            }
        };
        Self::apply_field_edit(
            bot,
            dialogue,
            executors,
            msg,
            repository_id,
            EditField::Url,
            new_value,
        )
        .await
    }

    async fn apply_field_edit(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        repository_id: i32,
        field: EditField,
        new_value: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let repo = match executors
            .commands
            .create_repository
            .repository_repo
            .find_by_id(RepositoryId(repository_id))
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, repository_id = repository_id, "Repository not found for edit");
                bot.send_message(
                    msg.chat.id,
                    t!("telegram_bot.dialogues.admin.repository.not_found").to_string(),
                )
                .await?;
                dialogue.exit().await.ok();
                return Ok(());
            }
        };

        let cmd = match field {
            EditField::Name => UpdateRepositoryCommand {
                id: repo.id,
                name: new_value,
                owner: repo.owner,
                url: repo.url,
            },
            EditField::Owner => UpdateRepositoryCommand {
                id: repo.id,
                name: repo.name,
                owner: new_value,
                url: repo.url,
            },
            EditField::Url => UpdateRepositoryCommand {
                id: repo.id,
                name: repo.name,
                owner: repo.owner,
                url: new_value,
            },
        };

        let loading = bot
            .send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.admin.repository.edit.loading").to_string(),
            )
            .await?;

        match executors.commands.update_repository.execute(&cmd).await {
            Ok(r) => {
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    t!(
                        "telegram_bot.dialogues.admin.repository.edit.success",
                        owner = r.repository.owner,
                        name = r.repository.name
                    )
                    .to_string(),
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to update repository");
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    t!("telegram_bot.dialogues.admin.repository.edit.db_error").to_string(),
                )
                .await?;
            }
        }

        dialogue.exit().await.ok();
        Ok(())
    }
}
