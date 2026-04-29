use crate::application::user::queries::get_user_roles_by_telegram_id::query::GetUserRolesByTelegramIdQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::setup_notifications::TelegramBotSetupNotificationsState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub struct TelegramBotSetupNotificationsCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotSetupNotificationsCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executors: Arc<ApplicationBoostrapExecutors>,
        dialogue: Arc<TelegramBotDialogueType>,
    ) -> Self {
        Self {
            context,
            executors,
            dialogue,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let roles = match self
            .executors
            .queries
            .get_user_roles_by_telegram_id
            .execute(&GetUserRolesByTelegramIdQuery { social_user_id })
            .await
        {
            Ok(r) => r.roles,
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

        let repositories = self
            .executors
            .commands
            .set_repository_notifications_chat
            .repository_repo
            .find_all()
            .await
            .unwrap_or_default();

        if repositories.is_empty() {
            self.context
                .bot
                .send_message(
                    self.context.msg.chat.id,
                    t!("telegram_bot.commands.setup_notifications.no_repositories").to_string(),
                )
                .await?;
            return Ok(());
        }

        let buttons: Vec<Vec<InlineKeyboardButton>> = repositories
            .into_iter()
            .map(|r| {
                vec![InlineKeyboardButton::callback(
                    format!("{}/{}", r.owner, r.name),
                    r.id.0.to_string(),
                )]
            })
            .collect();

        self.dialogue
            .update(TelegramBotDialogueState::SetupNotifications(
                TelegramBotSetupNotificationsState::SelectRepository,
            ))
            .await?;

        self.context
            .bot
            .send_message(
                self.context.msg.chat.id,
                t!("telegram_bot.commands.setup_notifications.select_repository").to_string(),
            )
            .reply_markup(InlineKeyboardMarkup::new(buttons))
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
}
