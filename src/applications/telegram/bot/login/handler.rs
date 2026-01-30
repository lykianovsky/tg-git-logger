use crate::applications::telegram::bot::context::TelegramBotCommandContext;
use crate::domain::user::use_cases::create_by_telegram_user::CreateUserByTelegramUserUseCase;
use crate::domain::user::value_objects::UserTelegramId;
use std::sync::Arc;
use teloxide::prelude::{Message, Requester};
use teloxide::RequestError;

pub struct TelegramBotLoginCommandHandler {
    context: TelegramBotCommandContext
}

impl TelegramBotLoginCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext
    ) -> Self {
        Self {
            context
        }
    }

    pub async fn execute(&self) -> Result<Message, RequestError> {
        return match CreateUserByTelegramUserUseCase::new(
            Arc::clone(&self.context.application_state.repositories.user)
        ).execute(
            UserTelegramId(self.context.user.id.0 as i32),
            self.context.msg.chat.id.0,
            self.context.user.username.clone()
        ).await {
            Ok(user) => {
                self.context.bot.send_message(
                    self.context.msg.chat.id,
                    format!("You have been logged in successfully, user ID: {}", user.id.0),
                ).await
            },
            Err(e) => {
                    self.context.bot.send_message(
                        self.context.msg.chat.id,
                        format!("Failed to log in: {:?}", e),
                    ).await
                }
        }
    }
}