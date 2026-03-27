pub mod helpers;
pub mod modules;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::modules::repository::TelegramBotDialogueAdminRepositoryDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::task_tracker::TelegramBotDialogueAdminTaskTrackerDispatcher;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::admin::TelegramBotAdminAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::{Bot, dptree};

use crate::delivery::bot::telegram::keyboards::actions::admin_repository::TelegramBotAdminRepositoryAction;

/// Состояния административного диалога.
///
/// Структура:
///   Menu
///   ├── ConfigureRepository → меню репозитория
///   │     ├── Create: CreateRepository{Name,Owner,Url,ExternalId}
///   │     └── Edit:   EditRepository{Select,Menu,Name,Owner,Url,ExternalId}
///   └── ConfigureTaskTracker → TaskTracker{SelectRepository,...поля...}
#[derive(Debug, Clone, Default)]
pub enum TelegramBotDialogueAdminState {
    #[default]
    Menu,

    // ── Репозиторий ─────────────────────────────────────────────────────────
    ConfigureRepository,

    // Создание
    CreateRepositoryName,
    CreateRepositoryOwner {
        name: String,
    },
    CreateRepositoryFinish {
        name: String,
        owner: String,
    },

    // Редактирование
    EditRepositorySelect,
    EditRepositoryMenu {
        repository_id: i32,
    },
    EditRepositoryName {
        repository_id: i32,
    },
    EditRepositoryOwner {
        repository_id: i32,
    },
    EditRepositoryUrl {
        repository_id: i32,
    },

    // Удаление
    DeleteRepositorySelect,
    DeleteRepositoryConfirm {
        repository_id: i32,
    },

    // ── Таск-трекер ─────────────────────────────────────────────────────────
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
            .branch(TelegramBotDialogueAdminRepositoryDispatcher::query_branches())
            .branch(TelegramBotDialogueAdminTaskTrackerDispatcher::query_branches());

        let messages = Update::filter_message()
            .branch(TelegramBotDialogueAdminRepositoryDispatcher::message_branches())
            .branch(TelegramBotDialogueAdminTaskTrackerDispatcher::message_branches());

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
        let message_id = msg.id();

        match action {
            TelegramBotAdminAction::ConfigureRepository => {
                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ConfigureRepository,
                    ))
                    .await?;

                let keyboard = KeyboardBuilder::new()
                    .row::<TelegramBotAdminRepositoryAction>(vec![
                        TelegramBotAdminRepositoryAction::Create,
                        TelegramBotAdminRepositoryAction::Edit,
                    ])
                    .row::<TelegramBotAdminRepositoryAction>(vec![
                        TelegramBotAdminRepositoryAction::Delete,
                    ])
                    .build();

                bot.edit_message_text(chat_id, message_id, "📦 Репозитории:")
                    .reply_markup(keyboard)
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
                    bot.edit_message_text(
                        chat_id,
                        message_id,
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
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectRepository,
                    ))
                    .await?;

                bot.edit_message_text(
                    chat_id,
                    message_id,
                    "📦 Выберите репозиторий для таск-трекера:",
                )
                .reply_markup(keyboard)
                .await?;
            }
        }

        Ok(())
    }
}
