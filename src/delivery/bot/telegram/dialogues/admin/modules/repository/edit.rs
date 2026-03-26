use crate::application::repository::commands::update_repository::command::UpdateRepositoryCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::{TelegramBotDialogueState, TelegramBotDialogueType};
use crate::delivery::bot::telegram::keyboards::actions::admin_repository_edit_field::TelegramBotAdminRepositoryEditField;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::{dptree, Bot};

pub fn query_branches()
-> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
    dptree::entry()
        .branch(
            case![TelegramBotDialogueAdminState::EditRepositorySelect]
                .endpoint(handle_select),
        )
        .branch(
            case![TelegramBotDialogueAdminState::EditRepositoryMenu { repository_id }]
                .endpoint(handle_field_choice),
        )
}

pub fn message_branches()
-> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
    dptree::entry()
        .branch(
            case![TelegramBotDialogueAdminState::EditRepositoryName { repository_id }]
                .endpoint(handle_edit_name),
        )
        .branch(
            case![TelegramBotDialogueAdminState::EditRepositoryOwner { repository_id }]
                .endpoint(handle_edit_owner),
        )
        .branch(
            case![TelegramBotDialogueAdminState::EditRepositoryUrl { repository_id }]
                .endpoint(handle_edit_url),
        )
        .branch(
            case![TelegramBotDialogueAdminState::EditRepositoryExternalId { repository_id }]
                .endpoint(handle_edit_external_id),
        )
}

/// Callback: пользователь выбрал репозиторий (callback data = "repo_select_{id}")
async fn handle_select(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");
    let repository_id: i32 = match data.strip_prefix("repo_select_").and_then(|s| s.parse().ok()) {
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

    let repository = executors
        .commands
        .create_repository
        .repository_repo
        .find_by_id(RepositoryId(repository_id))
        .await;

    let current_info = match repository {
        Ok(r) => format!(
            "📦 <b>{}/{}</b>\n🔗 {}\n🔢 External ID: {}",
            r.owner, r.name, r.url, r.external_id
        ),
        Err(_) => "Текущие данные недоступны.".to_string(),
    };

    let keyboard = KeyboardBuilder::new()
        .row::<TelegramBotAdminRepositoryEditField>(vec![
            TelegramBotAdminRepositoryEditField::Name,
            TelegramBotAdminRepositoryEditField::Owner,
        ])
        .row::<TelegramBotAdminRepositoryEditField>(vec![
            TelegramBotAdminRepositoryEditField::Url,
            TelegramBotAdminRepositoryEditField::ExternalId,
        ])
        .build();

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::EditRepositoryMenu { repository_id },
        ))
        .await?;

    bot.send_message(
        msg.chat().id,
        format!("{}\n\nЧто редактируем?", current_info),
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

/// Callback: пользователь выбрал поле для редактирования
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
            "📝 Введите новое название репозитория:",
        ),
        TelegramBotAdminRepositoryEditField::Owner => (
            TelegramBotDialogueAdminState::EditRepositoryOwner { repository_id },
            "👤 Введите нового владельца (owner):",
        ),
        TelegramBotAdminRepositoryEditField::Url => (
            TelegramBotDialogueAdminState::EditRepositoryUrl { repository_id },
            "🔗 Введите новый URL репозитория:",
        ),
        TelegramBotAdminRepositoryEditField::ExternalId => (
            TelegramBotDialogueAdminState::EditRepositoryExternalId { repository_id },
            "🔢 Введите новый External ID (GitHub repo ID):",
        ),
    };

    dialogue
        .update(TelegramBotDialogueState::Admin(next_state))
        .await?;

    bot.send_message(msg.chat().id, prompt).await?;
    Ok(())
}

async fn handle_edit_name(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    repository_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let new_value = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
            return Ok(());
        }
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
            bot.send_message(msg.chat.id, format!("❌ Репозиторий не найден: {e}")).await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let cmd = UpdateRepositoryCommand {
        id: repo.id,
        external_id: repo.external_id,
        name: new_value,
        owner: repo.owner,
        url: repo.url,
    };

    save_and_respond(bot, dialogue, executors, msg, cmd).await
}

async fn handle_edit_owner(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    repository_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let new_value = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
            return Ok(());
        }
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
            bot.send_message(msg.chat.id, format!("❌ Репозиторий не найден: {e}")).await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let cmd = UpdateRepositoryCommand {
        id: repo.id,
        external_id: repo.external_id,
        name: repo.name,
        owner: new_value,
        url: repo.url,
    };

    save_and_respond(bot, dialogue, executors, msg, cmd).await
}

async fn handle_edit_url(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    repository_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let new_value = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
            return Ok(());
        }
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
            bot.send_message(msg.chat.id, format!("❌ Репозиторий не найден: {e}")).await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let cmd = UpdateRepositoryCommand {
        id: repo.id,
        external_id: repo.external_id,
        name: repo.name,
        owner: repo.owner,
        url: new_value,
    };

    save_and_respond(bot, dialogue, executors, msg, cmd).await
}

async fn handle_edit_external_id(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    repository_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => {
            bot.send_message(msg.chat.id, "❌ Введите числовое значение.").await?;
            return Ok(());
        }
    };

    let new_external_id: i64 = match text.parse() {
        Ok(v) => v,
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ Некорректное значение. Введите целое число.").await?;
            return Ok(());
        }
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
            bot.send_message(msg.chat.id, format!("❌ Репозиторий не найден: {e}")).await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let cmd = UpdateRepositoryCommand {
        id: repo.id,
        external_id: new_external_id,
        name: repo.name,
        owner: repo.owner,
        url: repo.url,
    };

    save_and_respond(bot, dialogue, executors, msg, cmd).await
}

async fn save_and_respond(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    cmd: UpdateRepositoryCommand,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match executors.commands.update_repository.execute(&cmd).await {
        Ok(r) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "✅ Репозиторий <b>{}/{}</b> обновлён.",
                    r.repository.owner,
                    r.repository.name
                ),
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to update repository");
            bot.send_message(msg.chat.id, format!("❌ Ошибка обновления: {e}")).await?;
        }
    }

    dialogue.exit().await.ok();
    Ok(())
}
