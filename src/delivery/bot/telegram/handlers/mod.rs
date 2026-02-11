mod register;
mod start;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::command::TelegramBotCommand;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::handlers::register::TelegramBotRegisterCommandHandler;
use crate::delivery::bot::telegram::handlers::start::TelegramBotStartCommandHandler;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::ResponseResult;
use teloxide::types::{Message, User};

pub async fn handle(
    bot: Bot,
    user: User,
    msg: Message,
    cmd: TelegramBotCommand,
    executors: Arc<ApplicationBoostrapExecutors>,
) -> ResponseResult<()> {
    let context = TelegramBotCommandContext {
        bot,
        user,
        msg,
        cmd,
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
            )
            .execute()
            .await?;
        }
    }

    Ok(())
}
