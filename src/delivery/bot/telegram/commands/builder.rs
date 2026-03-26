use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::commands::admin::TelegramBotAdminCommandHandler;
use crate::delivery::bot::telegram::commands::bind_repository::TelegramBotBindRepositoryCommandHandler;
use crate::delivery::bot::telegram::commands::register::TelegramBotRegisterCommandHandler;
use crate::delivery::bot::telegram::commands::report::TelegramBotVersionControlReportCommandHandler;
use crate::delivery::bot::telegram::commands::start::TelegramBotStartCommandHandler;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::prelude::Requester;
use teloxide::types::{Message, User};
use teloxide::Bot;

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
    let context = TelegramBotCommandContext {
        bot,
        user,
        msg,
        cmd,
        config,
    };

    if !matches!(context.msg.chat.kind, teloxide::types::ChatKind::Private(_)) {
        context
            .bot
            .send_message(
                context.msg.chat.id,
                "Эта команда доступна только в приватном чате.",
            )
            .await?;

        return Ok(());
    }

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
    }

    Ok(())
}
