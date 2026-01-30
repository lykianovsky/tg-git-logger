use crate::applications::state::ApplicationState;
use crate::applications::telegram::bot::commands::Command;
use std::sync::Arc;
use teloxide::prelude::Message;
use teloxide::types::User;
use teloxide::Bot;

pub struct TelegramBotCommandContext {
    pub bot: Arc<Bot>,
    pub user: Arc<User>,
    pub msg: Arc<Message>,
    pub cmd: Arc<Command>,
    pub application_state: Arc<ApplicationState>
}