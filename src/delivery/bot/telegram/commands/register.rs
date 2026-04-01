use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::registration::TelegramBotDialogueRegistrationState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::choose_role::TelegramBotChooseRoleAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub struct TelegramBotRegisterCommandHandler {
    context: TelegramBotCommandContext,
    executor: Arc<CreateOAuthLinkExecutor>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotRegisterCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executor: Arc<CreateOAuthLinkExecutor>,
        dialogue: Arc<TelegramBotDialogueType>,
    ) -> Self {
        Self {
            context,
            executor,
            dialogue,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotChooseRoleAction>(vec![
                TelegramBotChooseRoleAction::Developer,
                TelegramBotChooseRoleAction::QualityAssurance,
            ])
            .build();

        self.dialogue
            .update(TelegramBotDialogueState::Registration(
                TelegramBotDialogueRegistrationState::ChooseRole,
            ))
            .await?;

        self.context
            .bot
            .send_message(
                self.context.msg.chat.id,
                t!("telegram_bot.commands.register.choose_role").to_string(),
            )
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
