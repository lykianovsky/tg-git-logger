use crate::delivery::bot::telegram::command::TelegramBotCommand;
use teloxide::Bot;
use teloxide::prelude::Message;
use teloxide::types::User;

pub struct TelegramBotCommandContext {
    pub bot: Bot,
    pub user: User,
    pub msg: Message,
    pub cmd: TelegramBotCommand,
}
