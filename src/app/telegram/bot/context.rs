use crate::app::telegram::bot::commands::Command;
use crate::infrastructure::delivery::state::ApplicationState;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::Message;
use teloxide::types::User;

pub struct TelegramBotCommandContext {
    pub bot: Arc<Bot>,
    pub user: Arc<User>,
    pub msg: Arc<Message>,
    pub cmd: Arc<Command>,
    pub application_state: Arc<ApplicationState>,
}
