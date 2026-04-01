use crate::application::user::queries::get_user_roles_by_telegram_id::executor::GetUserRolesByTelegramIdExecutor;
use crate::application::user::queries::get_user_roles_by_telegram_id::query::GetUserRolesByTelegramIdQuery;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::admin::TelegramBotAdminAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub struct TelegramBotAdminCommandHandler {
    context: TelegramBotCommandContext,
    get_user_roles: Arc<GetUserRolesByTelegramIdExecutor>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotAdminCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        get_user_roles: Arc<GetUserRolesByTelegramIdExecutor>,
        dialogue: Arc<TelegramBotDialogueType>,
    ) -> Self {
        Self {
            context,
            get_user_roles,
            dialogue,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let roles = match self
            .get_user_roles
            .execute(&GetUserRolesByTelegramIdQuery { social_user_id })
            .await
        {
            Ok(response) => response.roles,
            Err(_) => {
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.access_denied").to_string(),
                    )
                    .await?;
                return Ok(());
            }
        };

        if !roles.contains(&RoleName::Admin) {
            self.context
                .bot
                .send_message(
                    self.context.msg.chat.id,
                    t!("telegram_bot.commands.access_denied").to_string(),
                )
                .await?;
            return Ok(());
        }

        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotAdminAction>(vec![TelegramBotAdminAction::ConfigureRepository])
            .row::<TelegramBotAdminAction>(vec![TelegramBotAdminAction::ConfigureTaskTracker])
            .row::<TelegramBotAdminAction>(vec![TelegramBotAdminAction::QueuesStats])
            .row::<TelegramBotAdminAction>(vec![TelegramBotAdminAction::HealthPings])
            .row::<TelegramBotAdminAction>(vec![TelegramBotAdminAction::ManageUsers])
            .build();

        self.dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::Menu,
            ))
            .await?;

        self.context
            .bot
            .send_message(self.context.msg.chat.id, t!("telegram_bot.commands.admin.panel_title").to_string())
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
