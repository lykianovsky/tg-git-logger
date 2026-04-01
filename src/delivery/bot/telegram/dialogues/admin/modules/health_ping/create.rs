use crate::application::health_ping::commands::create_health_ping::command::CreateHealthPingCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::helpers::{extract_text, parse_integer};
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminHealthPingCreateDispatcher;

impl TelegramBotDialogueAdminHealthPingCreateDispatcher {
    pub fn message_branches(
    ) -> Handler<
        'static,
        Result<(), Box<dyn std::error::Error + Send + Sync>>,
        DpHandlerDescription,
    > {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingCreateName]
                    .endpoint(handle_create_name),
            )
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingCreateUrl { name }]
                    .endpoint(handle_create_url),
            )
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingCreateInterval {
                    name,
                    url
                }]
                .endpoint(handle_create_interval),
            )
    }
}

async fn handle_create_name(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let name = match extract_text(&msg) {
        Some(t) => t,
        None => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.admin.health_ping.name_required").to_string(),
            )
            .await?;

            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::HealthPingCreateUrl { name },
        ))
        .await?;

    bot.send_message(
        msg.chat.id,
        t!("telegram_bot.dialogues.admin.health_ping.enter_url").to_string(),
    )
    .await?;

    Ok(())
}

async fn handle_create_url(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    name: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = match extract_text(&msg) {
        Some(t) => t,
        None => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.admin.health_ping.url_required").to_string(),
            )
            .await?;

            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::HealthPingCreateInterval { name, url },
        ))
        .await?;

    bot.send_message(
        msg.chat.id,
        t!("telegram_bot.dialogues.admin.health_ping.enter_interval").to_string(),
    )
    .await?;

    Ok(())
}

async fn handle_create_interval(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    (name, url): (String, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let interval = match parse_integer(&msg) {
        Some(i) if i > 0 => i,
        _ => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.admin.health_ping.interval_required")
                    .to_string(),
            )
            .await?;

            return Ok(());
        }
    };

    let cmd = CreateHealthPingCommand {
        name: name.clone(),
        url,
        interval_minutes: interval,
    };

    let reply = match executors.commands.create_health_ping.execute(&cmd).await {
        Ok(_) => t!(
            "telegram_bot.dialogues.admin.health_ping.created",
            name = name
        )
        .to_string(),

        Err(e) => {
            tracing::error!(error = %e, "Failed to create health ping");
            t!("telegram_bot.dialogues.admin.health_ping.create_error").to_string()
        }
    };

    bot.send_message(msg.chat.id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}
