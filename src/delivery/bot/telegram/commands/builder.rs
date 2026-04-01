use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::commands::admin::TelegramBotAdminCommandHandler;
use crate::delivery::bot::telegram::commands::digest::TelegramBotDigestCommandHandler;
use crate::delivery::bot::telegram::commands::bind_repository::TelegramBotBindRepositoryCommandHandler;
use crate::delivery::bot::telegram::commands::register::TelegramBotRegisterCommandHandler;
use crate::delivery::bot::telegram::commands::report::TelegramBotVersionControlReportCommandHandler;
use crate::delivery::bot::telegram::commands::setup_webhook::TelegramBotSetupWebhookCommandHandler;
use crate::delivery::bot::telegram::commands::start::TelegramBotStartCommandHandler;
use crate::delivery::bot::telegram::commands::task::TelegramBotTaskCommandHandler;
use crate::delivery::bot::telegram::commands::unregister::TelegramBotUnregisterCommandHandler;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::macros::BotCommands;
use teloxide::prelude::Requester;
use teloxide::types::{ChatKind, Message, User};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum TelegramBotCommand {
    #[command(description = "Запустить бота")]
    Start,
    #[command(description = "Создать пользователя")]
    Register,
    #[command(description = "Получить отчет")]
    Report,
    #[command(
        rename = "bind_repository",
        description = "Привязать/отвязать репозиторий"
    )]
    BindRepository,
    #[command(description = "Панель администратора")]
    Admin,
    #[command(description = "Получить карточку по ID: /task 12345")]
    Task(String),
    #[command(
        rename = "setup_webhook",
        description = "Привязать чат к уведомлениям репозитория"
    )]
    SetupWebhook,

    #[command(description = "Деактивировать аккаунт")]
    Unregister,

    #[command(description = "Настройка дайджест-уведомлений")]
    Digest,
}

pub async fn handle(
    bot: Bot,
    user: User,
    msg: Message,
    cmd: TelegramBotCommand,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let TelegramBotCommand::SetupWebhook = &cmd {
        if !matches!(msg.chat.kind, ChatKind::Public(_)) {
            bot.send_message(msg.chat.id, t!("telegram_bot.commands.group_only").to_string())
                .await?;
            return Ok(());
        }

        let context = TelegramBotCommandContext {
            bot,
            user,
            msg,
            cmd,
            config,
        };
        return TelegramBotSetupWebhookCommandHandler::new(
            context,
            executors.clone(),
            Arc::new(dialogue),
        )
        .execute()
        .await;
    }

    // All other commands are private-chat only
    if !matches!(msg.chat.kind, ChatKind::Private(_)) {
        bot.send_message(msg.chat.id, t!("telegram_bot.commands.private_only").to_string())
            .await?;
        return Ok(());
    }

    let context = TelegramBotCommandContext {
        bot,
        user,
        msg,
        cmd,
        config,
    };

    match context.cmd {
        TelegramBotCommand::Start => {
            TelegramBotStartCommandHandler::new(context)
                .execute()
                .await?;
        }
        TelegramBotCommand::Register => {
            TelegramBotRegisterCommandHandler::new(
                context,
                executors.commands.create_oauth_link.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }
        TelegramBotCommand::Report => {
            TelegramBotVersionControlReportCommandHandler::new(context, Arc::new(dialogue))
                .execute()
                .await?;
        }
        TelegramBotCommand::BindRepository => {
            TelegramBotBindRepositoryCommandHandler::new(
                context,
                executors.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }
        TelegramBotCommand::Admin => {
            TelegramBotAdminCommandHandler::new(
                context,
                executors.queries.get_user_roles_by_telegram_id.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }
        TelegramBotCommand::Task(raw_id) => {
            TelegramBotTaskCommandHandler::new(
                context.bot,
                context.msg,
                executors.queries.get_task_card.clone(),
                raw_id,
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::Unregister => {
            TelegramBotUnregisterCommandHandler::new(
                context,
                executors.commands.deactivate_user.clone(),
            )
            .execute()
            .await?;
        }

        TelegramBotCommand::Digest => {
            TelegramBotDigestCommandHandler::new(
                context,
                executors.clone(),
                Arc::new(dialogue),
            )
            .execute()
            .await?;
        }

        // Handled above before private-chat guard
        TelegramBotCommand::SetupWebhook => {}
    }

    Ok(())
}
