use teloxide::prelude::*;
use teloxide::requests::ResponseResult;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum Command {
    #[command(description = "Запустить бота")]
    Start,

    #[command(description = "Привязать GitHub аккаунт")]
    Bind,

    #[command(description = "Показать помощь")]
    Help,
}

pub async fn handle(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "Привет 👋\nЯ слежу за Pull Request'ами и напомню, если их забыли проверить.",
            )
            .await?;
        }

        Command::Bind => {
            bot.send_message(
                msg.chat.id,
                "Отправь свой GitHub username в формате:\n/bind your_github_login",
            )
            .await?;
        }

        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
    }

    Ok(())
}
