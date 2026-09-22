use crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::tests::TelegramBotTestsState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub struct TelegramBotTestsCommandHandler {
    context: TelegramBotCommandContext,
    dialogue: Arc<TelegramBotDialogueType>,
    executors: Arc<ApplicationBoostrapExecutors>,
}

impl TelegramBotTestsCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        dialogue: Arc<TelegramBotDialogueType>,
        executors: Arc<ApplicationBoostrapExecutors>,
    ) -> Self {
        Self {
            context,
            dialogue,
            executors,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let repositories = match self
            .executors
            .queries
            .get_user_bound_repositories
            .execute(&GetUserBoundRepositoriesQuery { social_user_id })
            .await
        {
            Ok(response) => response.repositories,
            Err(error) => {
                tracing::error!(%error, "Failed to get user bound repositories");

                self.send_plain(t!("telegram_bot.dialogues.tests.load_repos_error").to_string())
                    .await?;

                return Ok(());
            }
        };

        if repositories.is_empty() {
            self.send_plain(t!("telegram_bot.dialogues.tests.no_bound_repos").to_string())
                .await?;

            return Ok(());
        }

        // Репозиторий выбирается всегда: карточка должна быть о конкретной репе
        let rows: Vec<Vec<InlineKeyboardButton>> = repositories
            .iter()
            .map(|repository| {
                vec![InlineKeyboardButton::callback(
                    format!("{}/{}", repository.owner, repository.name),
                    repository.id.0.to_string(),
                )]
            })
            .collect();

        self.dialogue
            .update(TelegramBotDialogueState::Tests(
                TelegramBotTestsState::SelectRepository,
            ))
            .await?;

        self.context
            .bot
            .send_message(
                self.context.msg.chat.id,
                t!("telegram_bot.dialogues.tests.select_repository").to_string(),
            )
            .reply_markup(InlineKeyboardMarkup::new(rows))
            .await?;

        Ok(())
    }

    async fn send_plain(
        &self,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.context
            .bot
            .send_message(self.context.msg.chat.id, text)
            .await?;

        Ok(())
    }
}
