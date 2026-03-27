use crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery;
use crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::bind_repository::TelegramBotBindRepositoryState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::collections::HashSet;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub struct TelegramBotBindRepositoryCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotBindRepositoryCommandHandler {
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

        let all_repos = self
            .executors
            .queries
            .get_all_repositories
            .execute(&GetAllRepositoriesQuery)
            .await;

        let all_repos = match all_repos {
            Ok(r) => r.repositories,
            Err(e) => {
                tracing::error!(error = %e, "Failed to get all repositories");
                self.context
                    .bot
                    .send_message(self.context.msg.chat.id, format!("❌ Ошибка: {e}"))
                    .await?;
                return Ok(());
            }
        };

        if all_repos.is_empty() {
            self.context
                .bot
                .send_message(self.context.msg.chat.id, "Нет доступных репозиториев.")
                .await?;
            return Ok(());
        }

        let bound_ids: HashSet<i32> = self
            .executors
            .queries
            .get_user_bound_repositories
            .execute(&GetUserBoundRepositoriesQuery { social_user_id })
            .await
            .map(|r| r.repositories.iter().map(|repo| repo.id.0).collect())
            .unwrap_or_default();

        let rows: Vec<Vec<InlineKeyboardButton>> = all_repos
            .iter()
            .map(|r| {
                let label = if bound_ids.contains(&r.id.0) {
                    format!("✅ {}/{}", r.owner, r.name)
                } else {
                    format!("{}/{}", r.owner, r.name)
                };
                vec![InlineKeyboardButton::callback(label, r.id.0.to_string())]
            })
            .collect();

        let keyboard = InlineKeyboardMarkup::new(rows);

        self.dialogue
            .update(TelegramBotDialogueState::BindRepository(
                TelegramBotBindRepositoryState::SelectRepository,
            ))
            .await?;

        self.context
            .bot
            .send_message(
                self.context.msg.chat.id,
                "📦 Выберите репозиторий для привязки/отвязки:\n(✅ — уже привязан)",
            )
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
