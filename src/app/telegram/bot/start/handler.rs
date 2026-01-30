use crate::app::telegram::bot::commands::Command;
use crate::app::telegram::bot::context::TelegramBotCommandContext;
use teloxide::RequestError;
use teloxide::prelude::{Message, Requester};
use teloxide::utils::command::BotCommands;

pub struct TelegramBotStartCommandHandler {
    context: TelegramBotCommandContext,
}

impl TelegramBotStartCommandHandler {
    pub fn new(context: TelegramBotCommandContext) -> Self {
        Self { context }
    }

    pub async fn execute(&self) -> Result<Message, RequestError> {
        self.context
            .bot
            .send_message(
                self.context.msg.chat.id,
                Command::descriptions().to_string(),
            )
            .await
    }
}
