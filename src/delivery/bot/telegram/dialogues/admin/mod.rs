pub mod modules;

use crate::application::repository::commands::create_repository::command::CreateRepositoryCommand;
use crate::application::repository::commands::create_repository_task_tracker::command::CreateRepositoryTaskTrackerCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::modules::configure_task_tracker_for_repository::TelegramBotDialogueAdminConfigureTaskTrackerForRepositoryDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::create_repository::TelegramBotDialogueAdminCreateRepositoryDispatcher;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use crate::delivery::bot::telegram::keyboards::actions::admin::TelegramBotAdminAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::{dptree, Bot};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotDialogueAdminState {
    #[default]
    Menu,

    // Создание репозитория
    CreateRepositoryName,
    CreateRepositoryOwner {
        name: String,
    },
    CreateRepositoryUrl {
        name: String,
        owner: String,
    },
    CreateRepositoryExternalId {
        name: String,
        owner: String,
        url: String,
    },

    // Настройка таск-трекера
    ConfigureTaskTrackerSelectRepository,
    ConfigureTaskTrackerSpaceId {
        repository_id: i32,
    },
    ConfigureTaskTrackerQaColumnId {
        repository_id: i32,
        space_id: i32,
    },
    ConfigureTaskTrackerExtractPattern {
        repository_id: i32,
        space_id: i32,
        qa_column_id: i32,
    },
    ConfigureTaskTrackerPathToCard {
        repository_id: i32,
        space_id: i32,
        qa_column_id: i32,
        extract_pattern: String,
    },
}

pub struct TelegramBotDialogueAdminDispatcher {}

impl TelegramBotDialogueAdminDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let callback_queries = Update::filter_callback_query()
            .branch(
                case![TelegramBotDialogueAdminState::Menu]
                    .endpoint(TelegramBotDialogueAdminDispatcher::handle_menu),
            )
            .branch(
                TelegramBotDialogueAdminConfigureTaskTrackerForRepositoryDispatcher::query_branches(
                ),
            );

        let messages = Update::filter_message()
            .branch(TelegramBotDialogueAdminCreateRepositoryDispatcher::message_branches())
            .branch(TelegramBotDialogueAdminConfigureTaskTrackerForRepositoryDispatcher::message_branches());

        dptree::entry().branch(callback_queries).branch(messages)
    }

    async fn handle_menu(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");

        let action = match TelegramBotAdminAction::from_callback_data(data) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!(error = %e, "Unknown admin menu action");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let chat_id = msg.chat().id;

        match action {
            TelegramBotAdminAction::CreateRepository => {
                dialogue
                    .update(
                        crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                            TelegramBotDialogueAdminState::CreateRepositoryName,
                        ),
                    )
                    .await?;

                bot.send_message(chat_id, "📝 Введите название репозитория:")
                    .await?;
            }
            TelegramBotAdminAction::ConfigureTaskTracker => {
                let repositories = executors
                    .commands
                    .create_repository
                    .repository_repo
                    .find_all()
                    .await
                    .unwrap_or_default();

                if repositories.is_empty() {
                    bot.send_message(
                        chat_id,
                        "❌ Нет доступных репозиториев. Сначала создайте репозиторий.",
                    )
                    .await?;
                    dialogue.exit().await.ok();
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

                let keyboard = InlineKeyboardMarkup::new(buttons);

                dialogue
                    .update(
                        crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                            TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectRepository,
                        ),
                    )
                    .await?;

                bot.send_message(chat_id, "📦 Выберите репозиторий:")
                    .reply_markup(keyboard)
                    .await?;
            }
        }

        Ok(())
    }
}
