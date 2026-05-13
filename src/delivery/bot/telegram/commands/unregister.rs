use crate::application::user::commands::deactivate_user::command::DeactivateUserCommand;
use crate::application::user::commands::deactivate_user::error::DeactivateUserExecutorError;
use crate::application::user::commands::deactivate_user::executor::DeactivateUserExecutor;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::prelude::Requester;

pub struct TelegramBotUnregisterCommandHandler {
    context: TelegramBotCommandContext,
    executor: Arc<DeactivateUserExecutor>,
}

impl TelegramBotUnregisterCommandHandler {
    pub fn new(context: TelegramBotCommandContext, executor: Arc<DeactivateUserExecutor>) -> Self {
        Self { context, executor }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let cmd = DeactivateUserCommand { social_user_id };

        let reply = match self.executor.execute(&cmd).await {
            Ok(response) => {
                if response.new_state {
                    t!("telegram_bot.commands.toggle_is_active.activated").to_string()
                } else {
                    t!("telegram_bot.commands.toggle_is_active.deactivated").to_string()
                }
            }

            Err(DeactivateUserExecutorError::SocialAccountNotFound) => {
                t!("telegram_bot.commands.toggle_is_active.not_registered").to_string()
            }

            Err(e) => {
                tracing::error!(error = %e, "Failed to toggle account status");
                t!("telegram_bot.commands.toggle_is_active.error").to_string()
            }
        };

        self.context
            .bot
            .send_message(self.context.msg.chat.id, reply)
            .await?;

        Ok(())
    }
}
