mod register;
mod report;
mod start;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::command::TelegramBotCommand;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::handlers::register::TelegramBotRegisterCommandHandler;
use crate::delivery::bot::telegram::handlers::report::TelegramBotWeeklyReportCommandHandler;
use crate::delivery::bot::telegram::handlers::start::TelegramBotStartCommandHandler;
use std::sync::Arc;
use teloxide::prelude::ResponseResult;
use teloxide::types::{Message, User};
use teloxide::Bot;

pub async fn handle(
    bot: Bot,
    user: User,
    msg: Message,
    cmd: TelegramBotCommand,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
) -> ResponseResult<()> {
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
            )
            .execute()
            .await?;
        }
        TelegramBotCommand::WeeklyReport => {
            TelegramBotWeeklyReportCommandHandler::new(
                context,
                executors.queries.build_weekly_report.clone(),
            )
            .execute()
            .await?;
        }
    }

    Ok(())
}
