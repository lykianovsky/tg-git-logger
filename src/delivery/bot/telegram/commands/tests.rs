use crate::application::test_run::queries::get_last_test_run::query::GetLastTestRunQuery;
use crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::tests::TelegramBotTestsState;
use crate::delivery::bot::telegram::dialogues::tests::card::{
    build_card_keyboard, build_card_text,
};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

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

        match repositories.len() {
            0 => {
                self.send_plain(t!("telegram_bot.dialogues.tests.no_bound_repos").to_string())
                    .await?
            }
            // Репозиторий один — выбирать нечего, сразу показываем карточку
            1 => self.send_card(repositories[0].id).await?,
            _ => {
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
            }
        }

        Ok(())
    }

    async fn send_card(
        &self,
        repository_id: RepositoryId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let run = self
            .executors
            .queries
            .get_last_test_run
            .execute(&GetLastTestRunQuery { repository_id })
            .await
            .map(|response| response.run)
            .unwrap_or_default();

        self.dialogue
            .update(TelegramBotDialogueState::Tests(
                TelegramBotTestsState::Card {
                    repository_id: repository_id.0,
                },
            ))
            .await?;

        self.context
            .bot
            .send_message(self.context.msg.chat.id, build_card_text(run.as_ref()))
            .parse_mode(ParseMode::Html)
            .reply_markup(build_card_keyboard(run.as_ref()))
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
