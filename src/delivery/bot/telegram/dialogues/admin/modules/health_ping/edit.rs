use crate::application::health_ping::commands::update_health_ping::command::UpdateHealthPingCommand;
use crate::application::health_ping::queries::get_all_health_pings::query::GetAllHealthPingsQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::helpers::{extract_text, parse_integer};
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::admin_health_ping_edit::TelegramBotAdminHealthPingEditAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminHealthPingEditDispatcher;

impl TelegramBotDialogueAdminHealthPingEditDispatcher {
    pub fn query_branches(
    ) -> Handler<
        'static,
        Result<(), Box<dyn std::error::Error + Send + Sync>>,
        DpHandlerDescription,
    > {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingEditSelect]
                    .endpoint(handle_edit_select),
            )
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingEditMenu { ping_id }]
                    .endpoint(handle_edit_menu),
            )
    }

    pub fn message_branches(
    ) -> Handler<
        'static,
        Result<(), Box<dyn std::error::Error + Send + Sync>>,
        DpHandlerDescription,
    > {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingEditName { ping_id }]
                    .endpoint(handle_edit_name),
            )
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingEditUrl { ping_id }]
                    .endpoint(handle_edit_url),
            )
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingEditInterval { ping_id }]
                    .endpoint(handle_edit_interval),
            )
    }
}

async fn handle_edit_select(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    query: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let ping_id: i32 = match data.parse() {
        Ok(v) => v,
        Err(_) => {
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let keyboard = KeyboardBuilder::new()
        .row::<TelegramBotAdminHealthPingEditAction>(vec![
            TelegramBotAdminHealthPingEditAction::Name,
        ])
        .row::<TelegramBotAdminHealthPingEditAction>(vec![
            TelegramBotAdminHealthPingEditAction::Url,
        ])
        .row::<TelegramBotAdminHealthPingEditAction>(vec![
            TelegramBotAdminHealthPingEditAction::Interval,
        ])
        .row::<TelegramBotAdminHealthPingEditAction>(vec![
            TelegramBotAdminHealthPingEditAction::Toggle,
        ])
        .row::<TelegramBotAdminHealthPingEditAction>(vec![
            TelegramBotAdminHealthPingEditAction::Delete,
        ])
        .build();

    dialogue
        .update(TelegramBotDialogueState::Admin(
            TelegramBotDialogueAdminState::HealthPingEditMenu { ping_id },
        ))
        .await?;

    bot.send_message(
        msg.chat().id,
        t!("telegram_bot.dialogues.admin.health_ping.select_for_edit").to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

async fn handle_edit_menu(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    ping_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let action = match TelegramBotAdminHealthPingEditAction::from_callback_data(data) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, "Unknown health ping edit action");
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    match action {
        TelegramBotAdminHealthPingEditAction::Name => {
            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::HealthPingEditName { ping_id },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.health_ping.enter_name").to_string(),
            )
            .await?;
        }

        TelegramBotAdminHealthPingEditAction::Url => {
            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::HealthPingEditUrl { ping_id },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.health_ping.enter_url").to_string(),
            )
            .await?;
        }

        TelegramBotAdminHealthPingEditAction::Interval => {
            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::HealthPingEditInterval { ping_id },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.health_ping.enter_interval")
                    .to_string(),
            )
            .await?;
        }

        TelegramBotAdminHealthPingEditAction::Toggle => {
            let ping = executors
                .queries
                .get_all_health_pings
                .execute(&GetAllHealthPingsQuery)
                .await
                .ok()
                .and_then(|r| r.pings.into_iter().find(|p| p.id.0 == ping_id));

            if let Some(ping) = ping {
                let cmd = UpdateHealthPingCommand {
                    id: HealthPingId(ping_id),
                    name: None,
                    url: None,
                    interval_minutes: None,
                    is_active: Some(!ping.is_active),
                };

                match executors.commands.update_health_ping.execute(&cmd).await {
                    Ok(_) => {
                        let reply = if !ping.is_active {
                            t!("telegram_bot.dialogues.admin.health_ping.enabled")
                        } else {
                            t!("telegram_bot.dialogues.admin.health_ping.disabled")
                        };

                        bot.send_message(msg.chat().id, reply.to_string())
                            .await?;
                    }

                    Err(e) => {
                        tracing::error!(error = %e, "Failed to toggle health ping");

                        bot.send_message(
                            msg.chat().id,
                            t!(
                                "telegram_bot.dialogues.admin.health_ping.update_error"
                            )
                            .to_string(),
                        )
                        .await?;
                    }
                }
            }

            dialogue.exit().await.ok();
        }

        TelegramBotAdminHealthPingEditAction::Delete => {
            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::HealthPingDeleteConfirm { ping_id },
                ))
                .await?;

            let keyboard = KeyboardBuilder::new()
                .row::<crate::delivery::bot::telegram::keyboards::actions::confirm::TelegramBotConfirmAction>(vec![
                    crate::delivery::bot::telegram::keyboards::actions::confirm::TelegramBotConfirmAction::Yes,
                    crate::delivery::bot::telegram::keyboards::actions::confirm::TelegramBotConfirmAction::No,
                ])
                .build();

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.health_ping.confirm_delete")
                    .to_string(),
            )
            .reply_markup(keyboard)
            .await?;
        }
    }

    Ok(())
}

async fn handle_edit_name(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    ping_id: i32,
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

    let cmd = UpdateHealthPingCommand {
        id: HealthPingId(ping_id),
        name: Some(name),
        url: None,
        interval_minutes: None,
        is_active: None,
    };

    let reply = match executors.commands.update_health_ping.execute(&cmd).await {
        Ok(_) => {
            t!("telegram_bot.dialogues.admin.health_ping.updated").to_string()
        }

        Err(e) => {
            tracing::error!(error = %e, "Failed to update health ping");
            t!("telegram_bot.dialogues.admin.health_ping.update_error").to_string()
        }
    };

    bot.send_message(msg.chat.id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}

async fn handle_edit_url(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    ping_id: i32,
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

    let cmd = UpdateHealthPingCommand {
        id: HealthPingId(ping_id),
        name: None,
        url: Some(url),
        interval_minutes: None,
        is_active: None,
    };

    let reply = match executors.commands.update_health_ping.execute(&cmd).await {
        Ok(_) => {
            t!("telegram_bot.dialogues.admin.health_ping.updated").to_string()
        }

        Err(e) => {
            tracing::error!(error = %e, "Failed to update health ping");
            t!("telegram_bot.dialogues.admin.health_ping.update_error").to_string()
        }
    };

    bot.send_message(msg.chat.id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}

async fn handle_edit_interval(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    ping_id: i32,
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

    let cmd = UpdateHealthPingCommand {
        id: HealthPingId(ping_id),
        name: None,
        url: None,
        interval_minutes: Some(interval),
        is_active: None,
    };

    let reply = match executors.commands.update_health_ping.execute(&cmd).await {
        Ok(_) => {
            t!("telegram_bot.dialogues.admin.health_ping.updated").to_string()
        }

        Err(e) => {
            tracing::error!(error = %e, "Failed to update health ping");
            t!("telegram_bot.dialogues.admin.health_ping.update_error").to_string()
        }
    };

    bot.send_message(msg.chat.id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}
