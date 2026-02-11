use crate::delivery::bot::telegram::command::TelegramBotCommand;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use teloxide::RequestError;
use teloxide::prelude::Requester;
use teloxide::types::Message;
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
                TelegramBotCommand::descriptions().to_string(),
            )
            .await
    }
}
