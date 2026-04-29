use crate::application::health_ping::commands::delete_health_ping::command::DeleteHealthPingCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::confirm::TelegramBotConfirmAction;
use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::{Bot, dptree};

pub struct TelegramBotDialogueAdminHealthPingDeleteDispatcher;

impl TelegramBotDialogueAdminHealthPingDeleteDispatcher {
    pub fn query_branches()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        dptree::entry().branch(
            case![TelegramBotDialogueAdminState::HealthPingDeleteConfirm { ping_id }]
                .endpoint(handle_delete_confirm),
        )
    }
}

async fn handle_delete_confirm(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    ping_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let action = match TelegramBotConfirmAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => {
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    match action {
        TelegramBotConfirmAction::Yes => {
            let cmd = DeleteHealthPingCommand {
                id: HealthPingId(ping_id),
            };

            let reply = match executors.commands.delete_health_ping.execute(&cmd).await {
                Ok(_) => t!("telegram_bot.dialogues.admin.health_ping.deleted").to_string(),

                Err(e) => {
                    tracing::error!(error = %e, "Failed to delete health ping");
                    t!("telegram_bot.dialogues.admin.health_ping.update_error").to_string()
                }
            };

            bot.send_message(msg.chat().id, reply).await?;
        }

        TelegramBotConfirmAction::No => {
            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.common.cancelled").to_string(),
            )
            .await?;
        }
    }

    dialogue.exit().await.ok();

    Ok(())
}
