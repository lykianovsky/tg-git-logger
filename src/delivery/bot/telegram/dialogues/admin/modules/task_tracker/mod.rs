use crate::application::repository::commands::update_repository_task_tracker::command::UpdateRepositoryTaskTrackerCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::{TelegramBotDialogueState, TelegramBotDialogueType};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::Bot;

pub struct TelegramBotDialogueAdminTaskTrackerDispatcher {}

impl TelegramBotDialogueAdminTaskTrackerDispatcher {
    pub fn query_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry().branch(
            case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectRepository]
                .endpoint(handle_select_repository),
        )
    }

    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSpaceId { repository_id }]
                    .endpoint(handle_space_id),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerQaColumnId {
                    repository_id,
                    space_id
                }]
                .endpoint(handle_qa_column_id),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerExtractPattern {
                    repository_id,
                    space_id,
                    qa_column_id
                }]
                .endpoint(handle_extract_pattern),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerPathToCard {
                    repository_id,
                    space_id,
                    qa_column_id,
                    extract_pattern
                }]
                .endpoint(handle_path_to_card),
            )
    }
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

    bot.send_message(msg.chat().id, "🏢 Введите ID пространства (space_id):").await?;
    Ok(())
}

async fn handle_space_id(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    repository_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let space_id: i32 = match msg.text().and_then(|t| t.trim().parse().ok()) {
        Some(v) => v,
        None => {
            bot.send_message(msg.chat.id, "❌ Введите целое число.").await?;
            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::ConfigureTaskTrackerQaColumnId { repository_id, space_id },
        ))
        .await?;

    bot.send_message(msg.chat.id, "📋 Введите ID колонки QA (qa_column_id):").await?;
    Ok(())
}

async fn handle_qa_column_id(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (repository_id, space_id): (i32, i32),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let qa_column_id: i32 = match msg.text().and_then(|t| t.trim().parse().ok()) {
        Some(v) => v,
        None => {
            bot.send_message(msg.chat.id, "❌ Введите целое число.").await?;
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
        "🔍 Введите regex-паттерн для извлечения задачи (extract_pattern_regexp):",
    )
    .await?;
    Ok(())
}

async fn handle_extract_pattern(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (repository_id, space_id, qa_column_id): (i32, i32, i32),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let extract_pattern = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
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

    bot.send_message(msg.chat.id, "🗂 Введите путь к карточке (path_to_card):").await?;
    Ok(())
}

async fn handle_path_to_card(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    (repository_id, space_id, qa_column_id, extract_pattern): (i32, i32, i32, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path_to_card = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
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

    match executors
        .commands
        .update_repository_task_tracker
        .execute(&cmd)
        .await
    {
        Ok(r) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "✅ Таск-трекер для репозитория ID <b>{}</b> сохранён.",
                    r.tracker.repository_id.0
                ),
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to configure task tracker");
            bot.send_message(msg.chat.id, format!("❌ Ошибка: {e}")).await?;
        }
    }

    dialogue.exit().await.ok();
    Ok(())
}
