use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::command::TelegramBotCommand;
use std::sync::Arc;
use teloxide::prelude::Message;
use teloxide::types::User;
use teloxide::Bot;

pub struct TelegramBotCommandContext {
    pub bot: Bot,
    pub user: User,
    pub msg: Message,
    pub cmd: TelegramBotCommand,
    pub config: Arc<ApplicationConfig>,
}
