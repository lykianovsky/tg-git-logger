use crate::application::repository::commands::create_repository::command::CreateRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::{TelegramBotDialogueState, TelegramBotDialogueType};
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::{dptree, Bot};

pub fn message_branches()
-> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
    dptree::entry()
        .branch(
            case![TelegramBotDialogueAdminState::CreateRepositoryName]
                .endpoint(handle_name),
        )
        .branch(
            case![TelegramBotDialogueAdminState::CreateRepositoryOwner { name }]
                .endpoint(handle_owner),
        )
        .branch(
            case![TelegramBotDialogueAdminState::CreateRepositoryUrl { name, owner }]
                .endpoint(handle_url),
        )
        .branch(
            case![TelegramBotDialogueAdminState::CreateRepositoryExternalId { name, owner, url }]
                .endpoint(handle_external_id),
        )
}

async fn handle_name(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое название.").await?;
            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::CreateRepositoryOwner { name },
        ))
        .await?;

    bot.send_message(msg.chat.id, "👤 Введите владельца (owner):").await?;
    Ok(())
}

async fn handle_owner(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    name: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let owner = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::CreateRepositoryUrl { name, owner },
        ))
        .await?;

    bot.send_message(msg.chat.id, "🔗 Введите URL репозитория:").await?;
    Ok(())
}

async fn handle_url(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (name, owner): (String, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::CreateRepositoryExternalId { name, owner, url },
        ))
        .await?;

    bot.send_message(msg.chat.id, "🔢 Введите внешний ID репозитория (GitHub repo ID):").await?;
    Ok(())
}

async fn handle_external_id(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    (name, owner, url): (String, String, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите числовое значение.").await?;
            return Ok(());
        }
    };

    let external_id: i64 = match text.parse() {
        Ok(v) => v,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Некорректное значение. Введите целое число.").await?;
            return Ok(());
        }
    };

    let loading = bot.send_message(msg.chat.id, "Создаём репозиторий...").await?;

    match executors
        .commands
        .create_repository
        .execute(&CreateRepositoryCommand { external_id, name, owner, url })
        .await
    {
        Ok(r) => {
            bot.edit_message_text(
                msg.chat.id,
                loading.id,
                format!(
                    "✅ Репозиторий <b>{}/{}</b> создан (ID: {}).",
                    r.repository.owner,
                    r.repository.name,
                    r.repository.id.0
                ),
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create repository");
            bot.edit_message_text(msg.chat.id, loading.id, format!("❌ Ошибка: {e}")).await?;
        }
    }

    dialogue.exit().await.ok();
    Ok(())
}
