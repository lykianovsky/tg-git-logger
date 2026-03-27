use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::commands::builder::TelegramBotCommand;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::{CallbackQuery, Message};
use teloxide::types::User;

pub struct TelegramBotCommandContext {
    pub bot: Bot,
    pub user: User,
    pub msg: Message,
    pub cmd: TelegramBotCommand,
    pub config: Arc<ApplicationConfig>,
}

pub struct TelegramBotCallbackContext {
    pub bot: Bot,
    pub query: CallbackQuery,
}
