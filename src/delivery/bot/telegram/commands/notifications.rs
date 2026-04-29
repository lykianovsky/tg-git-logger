use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::notifications::{
    TelegramBotNotificationsState, send_main_menu,
};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;

pub struct TelegramBotNotificationsCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
    shared: Arc<ApplicationSharedDependency>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotNotificationsCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executors: Arc<ApplicationBoostrapExecutors>,
        shared: Arc<ApplicationSharedDependency>,
        dialogue: Arc<TelegramBotDialogueType>,
    ) -> Self {
        Self {
            context,
            executors,
            shared,
            dialogue,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        self.dialogue
            .update(TelegramBotDialogueState::Notifications(
                TelegramBotNotificationsState::Menu,
            ))
            .await?;

        send_main_menu(
            &self.context.bot,
            self.context.msg.chat.id,
            &self.executors,
            &self.shared,
            &self.context.config,
            social_user_id,
        )
        .await?;

        Ok(())
    }
}
