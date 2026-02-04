use crate::applications::state::ApplicationState;
use crate::applications::telegram::bot::bind::handler::TelegramBotBindCommandHandler;
use crate::applications::telegram::bot::context::TelegramBotCommandContext;
use crate::applications::telegram::bot::login::handler::TelegramBotLoginCommandHandler;
use crate::applications::telegram::bot::start::handler::TelegramBotStartCommandHandler;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::requests::ResponseResult;
use teloxide::types::User;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum Command {
    #[command(description = "Запустить бота")]
    Start,
    #[command(description = "Авторизоваться")]
    Login,
    #[command(description = "Привязать GitHub аккаунт")]
    Bind,
}

impl Command {
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::Start => "start",
            Command::Login => "login",
            Command::Bind => "bind",
        }
    }
}


pub async fn handle(bot: Arc<Bot>, user: User, msg: Message, cmd: Command, application_state: Arc<ApplicationState>) -> ResponseResult<()> {
    let msg = Arc::new(msg);
    let cmd = Arc::new(cmd);
    let user = Arc::new(user);

    let context = TelegramBotCommandContext {
        bot: Arc::clone(&bot),
        user: Arc::clone(&user),
        msg: Arc::clone(&msg),
        cmd: Arc::clone(&cmd),
        application_state: Arc::clone(&application_state),
    };

    match *cmd {
        Command::Start => {
            TelegramBotStartCommandHandler::new(context).execute().await?;
        }
        Command::Bind => {
            TelegramBotBindCommandHandler::new(context).execute().await?;
        }
        Command::Login => {
            TelegramBotLoginCommandHandler::new(context).execute().await?;
        }
    }

    bot.set_my_commands(Command::bot_commands()).await?;

    Ok(())
}
