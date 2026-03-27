use crate::application::repository::commands::update_repository_task_tracker::command::UpdateRepositoryTaskTrackerCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::admin::helpers::{
    db_error_message, extract_text, parse_integer,
};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

pub struct TelegramBotDialogueAdminTaskTrackerDispatcher {}

impl TelegramBotDialogueAdminTaskTrackerDispatcher {
    pub fn query_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry().branch(
            case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectRepository]
                .endpoint(Self::handle_select_repository),
        )
    }

    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSpaceId { repository_id }]
                    .endpoint(Self::handle_space_id),
            )
            .branch(
                case![
                    TelegramBotDialogueAdminState::ConfigureTaskTrackerQaColumnId {
                        repository_id,
                        space_id
                    }
                ]
                .endpoint(Self::handle_qa_column_id),
            )
            .branch(
                case![
                    TelegramBotDialogueAdminState::ConfigureTaskTrackerExtractPattern {
                        repository_id,
                        space_id,
                        qa_column_id
                    }
                ]
                .endpoint(Self::handle_extract_pattern),
            )
            .branch(
                case![
                    TelegramBotDialogueAdminState::ConfigureTaskTrackerPathToCard {
                        repository_id,
                        space_id,
                        qa_column_id,
                        extract_pattern
                    }
                ]
                .endpoint(Self::handle_path_to_card),
            )
    }

    async fn handle_select_repository(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let repository_id: i32 = match data.parse() {
            Ok(v) => v,
            Err(_) => {
                tracing::error!(data = %data, "Invalid repository_id in callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerSpaceId { repository_id },
            ))
            .await?;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            "🏢 Введите ID пространства (space_id):\n\n<i>Числовой идентификатор вашего пространства в Kaiten.</i>",
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(InlineKeyboardMarkup::default())
        .await?;
        Ok(())
    }

    async fn handle_space_id(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let space_id: i32 = match parse_integer(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    "❌ Space ID должен быть целым числом. Например: <code>12345</code>",
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
                return Ok(());
            }
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerQaColumnId {
                    repository_id,
                    space_id,
                },
            ))
            .await?;

        bot.send_message(
            msg.chat.id,
            "📋 Введите ID колонки QA (qa_column_id):\n\n<i>Числовой идентификатор колонки QA на доске.</i>",
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
        Ok(())
    }

    async fn handle_qa_column_id(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        (repository_id, space_id): (i32, i32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let qa_column_id: i32 = match parse_integer(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(
                    msg.chat.id,
                    "❌ QA Column ID должен быть целым числом. Например: <code>67890</code>",
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
                return Ok(());
            }
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerExtractPattern {
                    repository_id,
                    space_id,
                    qa_column_id,
                },
            ))
            .await?;

        bot.send_message(
            msg.chat.id,
            "🔍 Введите regex-паттерн для извлечения ID задачи:\n\n<i>Паттерн должен содержать группу захвата для числового ID. Например: <code>ZB-(\\d+)</code></i>",
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
        Ok(())
    }

    async fn handle_extract_pattern(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        (repository_id, space_id, qa_column_id): (i32, i32, i32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let extract_pattern = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(msg.chat.id, "❌ Введите regex-паттерн текстом.")
                    .await?;
                return Ok(());
            }
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerPathToCard {
                    repository_id,
                    space_id,
                    qa_column_id,
                    extract_pattern,
                },
            ))
            .await?;

        bot.send_message(
            msg.chat.id,
            "🗂 Введите путь к карточке задачи:\n\n<i>Используйте <code>{id}</code> как плейсхолдер для ID задачи. Например: <code>/space/123/boards/card/{id}</code></i>",
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
        Ok(())
    }

    async fn handle_path_to_card(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        (repository_id, space_id, qa_column_id, extract_pattern): (i32, i32, i32, String),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path_to_card = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(msg.chat.id, "❌ Введите путь к карточке текстом.")
                    .await?;
                return Ok(());
            }
        };

        let cmd = UpdateRepositoryTaskTrackerCommand {
            repository_id: RepositoryId(repository_id),
            space_id,
            qa_column_id,
            extract_pattern_regexp: extract_pattern,
            path_to_card,
        };

        let loading = bot
            .send_message(msg.chat.id, "⏳ Сохраняем настройки...")
            .await?;

        match executors
            .commands
            .update_repository_task_tracker
            .execute(&cmd)
            .await
        {
            Ok(r) => {
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    format!(
                        "✅ Настройки таск-трекера для репозитория ID <b>{}</b> успешно сохранены.",
                        r.tracker.repository_id.0
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to configure task tracker");
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    db_error_message("сохранить настройки таск-трекера"),
                )
                .await?;
            }
        }

        dialogue.exit().await.ok();
        Ok(())
    }
}
