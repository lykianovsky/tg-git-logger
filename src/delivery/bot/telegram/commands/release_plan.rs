use crate::application::user::queries::get_user_roles_by_telegram_id::query::GetUserRolesByTelegramIdQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::release_plan::TelegramBotReleasePlanState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::{Message, User};

pub struct TelegramBotReleasePlanCommandHandler {
    bot: Bot,
    msg: Message,
    user: User,
    executors: Arc<ApplicationBoostrapExecutors>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotReleasePlanCommandHandler {
    pub fn new(
        bot: Bot,
        msg: Message,
        user: User,
        executors: Arc<ApplicationBoostrapExecutors>,
        dialogue: Arc<TelegramBotDialogueType>,
    ) -> Self {
        Self {
            bot,
            msg,
            user,
            executors,
            dialogue,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.user.id.0 as i32);

        let roles = match self
            .executors
            .queries
            .get_user_roles_by_telegram_id
            .execute(&GetUserRolesByTelegramIdQuery { social_user_id })
            .await
        {
            Ok(r) => r.roles,
            Err(_) => {
                self.bot
                    .send_message(
                        self.msg.chat.id,
                        t!("telegram_bot.commands.releases.not_registered").to_string(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let can_manage = roles.contains(&RoleName::Admin)
            || roles.contains(&RoleName::ProductManager);
        if !can_manage {
            self.bot
                .send_message(
                    self.msg.chat.id,
                    t!("telegram_bot.commands.access_denied").to_string(),
                )
                .await?;
            return Ok(());
        }

        self.dialogue
            .update(TelegramBotDialogueState::ReleasePlan(
                TelegramBotReleasePlanState::EnterDate,
            ))
            .await?;

        self.bot
            .send_message(
                self.msg.chat.id,
                t!("telegram_bot.dialogues.release_plan.enter_date").to_string(),
            )
            .await?;
        Ok(())
    }
}
