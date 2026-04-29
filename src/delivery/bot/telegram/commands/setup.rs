use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::onboarding::{
    TelegramBotOnboardingState, send_repos_menu,
};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::Message;

pub struct TelegramBotSetupCommandHandler {
    bot: Bot,
    msg: Message,
    executors: Arc<ApplicationBoostrapExecutors>,
    dialogue: Arc<TelegramBotDialogueType>,
    social_user_id: SocialUserId,
}

impl TelegramBotSetupCommandHandler {
    pub fn new(
        bot: Bot,
        msg: Message,
        executors: Arc<ApplicationBoostrapExecutors>,
        dialogue: Arc<TelegramBotDialogueType>,
        social_user_id: SocialUserId,
    ) -> Self {
        Self {
            bot,
            msg,
            executors,
            dialogue,
            social_user_id,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.bot
            .send_message(
                self.msg.chat.id,
                t!("telegram_bot.dialogues.onboarding.welcome").to_string(),
            )
            .await?;

        self.dialogue
            .update(TelegramBotDialogueState::Onboarding(
                TelegramBotOnboardingState::SelectRepositories,
            ))
            .await?;

        send_repos_menu(
            &self.bot,
            self.msg.chat.id,
            &self.executors,
            self.social_user_id,
        )
        .await?;

        Ok(())
    }
}
