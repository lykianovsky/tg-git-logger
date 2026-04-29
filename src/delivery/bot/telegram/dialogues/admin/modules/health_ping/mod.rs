mod create;
mod delete;
mod edit;

use crate::application::health_ping::queries::get_all_health_pings::query::GetAllHealthPingsQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::admin::modules::health_ping::create::TelegramBotDialogueAdminHealthPingCreateDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::health_ping::delete::TelegramBotDialogueAdminHealthPingDeleteDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::health_ping::edit::TelegramBotDialogueAdminHealthPingEditDispatcher;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::admin_health_ping::TelegramBotAdminHealthPingAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::shared::command::CommandExecutor;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode};
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminHealthPingDispatcher;

impl TelegramBotDialogueAdminHealthPingDispatcher {
    pub fn query_branches()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::HealthPingList].endpoint(handle_list_action),
            )
            .branch(TelegramBotDialogueAdminHealthPingEditDispatcher::query_branches())
            .branch(TelegramBotDialogueAdminHealthPingDeleteDispatcher::query_branches())
    }

    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        dptree::entry()
            .branch(TelegramBotDialogueAdminHealthPingCreateDispatcher::message_branches())
            .branch(TelegramBotDialogueAdminHealthPingEditDispatcher::message_branches())
    }

    pub async fn show_list(
        bot: &Bot,
        chat_id: ChatId,
        message_id: MessageId,
        executors: &ApplicationBoostrapExecutors,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pings = executors
            .queries
            .get_all_health_pings
            .execute(&GetAllHealthPingsQuery)
            .await;

        let pings = match pings {
            Ok(r) => r.pings,
            Err(e) => {
                tracing::error!(error = %e, "Failed to get health pings");

                bot.edit_message_text(
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.admin.health_ping.load_error").to_string(),
                )
                .await?;

                return Ok(());
            }
        };

        let mut builder = MessageBuilder::new()
            .bold(&t!("telegram_bot.dialogues.admin.health_ping.title").to_string())
            .empty_line();

        if pings.is_empty() {
            builder =
                builder.line(&t!("telegram_bot.dialogues.admin.health_ping.empty").to_string());
        } else {
            for ping in &pings {
                let status_icon = match ping.last_status.as_deref() {
                    Some("ok") => "🟢",
                    Some("error") => "🔴",
                    _ => "⚪",
                };

                let active_icon = if ping.is_active { "✅" } else { "⏸" };

                let ms = ping
                    .last_response_ms
                    .map(|ms| format!(" ({ms}ms)"))
                    .unwrap_or_default();

                let line = format!(
                    "{} {} <b>{}</b> — {}мин{}",
                    status_icon, active_icon, ping.name, ping.interval_minutes, ms,
                );

                builder = builder.raw(&line).raw("\n");
            }
        }

        let text = builder.build();

        let mut keyboard = KeyboardBuilder::new().row::<TelegramBotAdminHealthPingAction>(vec![
            TelegramBotAdminHealthPingAction::Create,
        ]);

        if !pings.is_empty() {
            keyboard = keyboard.row::<TelegramBotAdminHealthPingAction>(vec![
                TelegramBotAdminHealthPingAction::Edit,
            ]);
        }

        keyboard = keyboard.row::<TelegramBotAdminHealthPingAction>(vec![
            TelegramBotAdminHealthPingAction::Cancel,
        ]);

        bot.edit_message_text(chat_id, message_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard.build())
            .await?;

        Ok(())
    }
}

async fn handle_list_action(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let action = match TelegramBotAdminHealthPingAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => {
            let msg = match query.message {
                Some(m) => m,
                None => return Ok(()),
            };

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.common.cancelled").to_string(),
            )
            .await?;

            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    match action {
        TelegramBotAdminHealthPingAction::Create => {
            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::HealthPingCreateName,
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.health_ping.enter_name").to_string(),
            )
            .await?;
        }

        TelegramBotAdminHealthPingAction::Edit => {
            let pings = executors
                .queries
                .get_all_health_pings
                .execute(&GetAllHealthPingsQuery)
                .await
                .map(|r| r.pings)
                .unwrap_or_default();

            if pings.is_empty() {
                bot.send_message(
                    msg.chat().id,
                    t!("telegram_bot.dialogues.admin.health_ping.empty").to_string(),
                )
                .await?;

                dialogue.exit().await.ok();
                return Ok(());
            }

            let rows: Vec<Vec<InlineKeyboardButton>> = pings
                .iter()
                .map(|p| {
                    vec![InlineKeyboardButton::callback(
                        p.name.clone(),
                        p.id.0.to_string(),
                    )]
                })
                .collect();

            dialogue
                .update(TelegramBotDialogueState::Admin(
                    TelegramBotDialogueAdminState::HealthPingEditSelect,
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.admin.health_ping.select_for_edit").to_string(),
            )
            .reply_markup(InlineKeyboardMarkup::new(rows))
            .await?;
        }

        TelegramBotAdminHealthPingAction::Cancel => {
            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.common.cancelled").to_string(),
            )
            .await?;

            dialogue.exit().await.ok();
        }
    }

    Ok(())
}
