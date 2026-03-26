use crate::application::repository::commands::create_repository::command::CreateRepositoryCommand;
use crate::application::repository::commands::create_repository_task_tracker::command::CreateRepositoryTaskTrackerCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
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
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectRepository]
                    .endpoint(
                        TelegramBotDialogueAdminDispatcher::handle_select_repository,
                    ),
            );

        let messages = Update::filter_message()
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryName]
                    .endpoint(TelegramBotDialogueAdminDispatcher::handle_create_repository_name),
            )
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryOwner { name }]
                    .endpoint(TelegramBotDialogueAdminDispatcher::handle_create_repository_owner),
            )
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryUrl { name, owner }]
                    .endpoint(TelegramBotDialogueAdminDispatcher::handle_create_repository_url),
            )
            .branch(
                case![TelegramBotDialogueAdminState::CreateRepositoryExternalId {
                    name,
                    owner,
                    url
                }]
                .endpoint(
                    TelegramBotDialogueAdminDispatcher::handle_create_repository_external_id,
                ),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSpaceId {
                    repository_id
                }]
                .endpoint(
                    TelegramBotDialogueAdminDispatcher::handle_configure_tracker_space_id,
                ),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerQaColumnId {
                    repository_id,
                    space_id
                }]
                .endpoint(
                    TelegramBotDialogueAdminDispatcher::handle_configure_tracker_qa_column_id,
                ),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerExtractPattern {
                    repository_id,
                    space_id,
                    qa_column_id
                }]
                .endpoint(
                    TelegramBotDialogueAdminDispatcher::handle_configure_tracker_extract_pattern,
                ),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerPathToCard {
                    repository_id,
                    space_id,
                    qa_column_id,
                    extract_pattern
                }]
                .endpoint(
                    TelegramBotDialogueAdminDispatcher::handle_configure_tracker_path_to_card,
                ),
            );

        dptree::entry().branch(callback_queries).branch(messages)
    }

    // --- Menu ---

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
                    .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::CreateRepositoryName,
                    ))
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
                    bot.send_message(chat_id, "❌ Нет доступных репозиториев. Сначала создайте репозиторий.")
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
                    .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectRepository,
                    ))
                    .await?;

                bot.send_message(chat_id, "📦 Выберите репозиторий:")
                    .reply_markup(keyboard)
                    .await?;
            }
        }

        Ok(())
    }

    // --- Create Repository steps ---

    async fn handle_create_repository_name(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let name = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое название.").await?;
                return Ok(());
            }
        };

        dialogue
            .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::CreateRepositoryOwner { name },
            ))
            .await?;

        bot.send_message(msg.chat.id, "👤 Введите владельца репозитория (owner):").await?;

        Ok(())
    }

    async fn handle_create_repository_owner(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let owner = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
                return Ok(());
            }
        };

        dialogue
            .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::CreateRepositoryUrl { name, owner },
            ))
            .await?;

        bot.send_message(msg.chat.id, "🔗 Введите URL репозитория:").await?;

        Ok(())
    }

    async fn handle_create_repository_url(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        (name, owner): (String, String),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
                return Ok(());
            }
        };

        dialogue
            .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::CreateRepositoryExternalId { name, owner, url },
            ))
            .await?;

        bot.send_message(msg.chat.id, "🔢 Введите внешний ID репозитория (GitHub repo ID):").await?;

        Ok(())
    }

    async fn handle_create_repository_external_id(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        (name, owner, url): (String, String, String),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите числовое значение.").await?;
                return Ok(());
            }
        };

        let external_id: i64 = match text.parse() {
            Ok(v) => v,
            Err(_) => {
                bot.send_message(msg.chat.id, "❌ Некорректное значение. Введите целое число.")
                    .await?;
                return Ok(());
            }
        };

        let cmd = CreateRepositoryCommand {
            external_id,
            name,
            owner,
            url,
        };

        match executors.commands.create_repository.execute(&cmd).await {
            Ok(response) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "✅ Репозиторий <b>{}/{}</b> успешно создан (ID: {}).",
                        response.repository.owner,
                        response.repository.name,
                        response.repository.id.0
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create repository");
                bot.send_message(msg.chat.id, format!("❌ Ошибка создания репозитория: {e}"))
                    .await?;
            }
        }

        dialogue.exit().await.ok();

        Ok(())
    }

    // --- Configure Task Tracker steps ---

    async fn handle_select_repository(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let repository_id: i32 = match data.parse() {
            Ok(v) => v,
            Err(_) => {
                tracing::error!(data = %data, "Invalid repository_id in callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        dialogue
            .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerSpaceId { repository_id },
            ))
            .await?;

        bot.send_message(msg.chat().id, "🏢 Введите ID пространства (space_id):").await?;

        Ok(())
    }

    async fn handle_configure_tracker_space_id(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите числовое значение.").await?;
                return Ok(());
            }
        };

        let space_id: i32 = match text.parse() {
            Ok(v) => v,
            Err(_) => {
                bot.send_message(msg.chat.id, "❌ Некорректное значение. Введите целое число.")
                    .await?;
                return Ok(());
            }
        };

        dialogue
            .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerQaColumnId {
                    repository_id,
                    space_id,
                },
            ))
            .await?;

        bot.send_message(msg.chat.id, "📋 Введите ID колонки QA (qa_column_id):").await?;

        Ok(())
    }

    async fn handle_configure_tracker_qa_column_id(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        (repository_id, space_id): (i32, i32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите числовое значение.").await?;
                return Ok(());
            }
        };

        let qa_column_id: i32 = match text.parse() {
            Ok(v) => v,
            Err(_) => {
                bot.send_message(msg.chat.id, "❌ Некорректное значение. Введите целое число.")
                    .await?;
                return Ok(());
            }
        };

        dialogue
            .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerExtractPattern {
                    repository_id,
                    space_id,
                    qa_column_id,
                },
            ))
            .await?;

        bot.send_message(
            msg.chat.id,
            "🔍 Введите регулярное выражение для извлечения задачи (extract_pattern_regexp):",
        )
        .await?;

        Ok(())
    }

    async fn handle_configure_tracker_extract_pattern(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        msg: Message,
        (repository_id, space_id, qa_column_id): (i32, i32, i32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let extract_pattern = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
                return Ok(());
            }
        };

        dialogue
            .update(crate::delivery::bot::telegram::dialogues::TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerPathToCard {
                    repository_id,
                    space_id,
                    qa_column_id,
                    extract_pattern,
                },
            ))
            .await?;

        bot.send_message(msg.chat.id, "🗂️ Введите путь к карточке (path_to_card):").await?;

        Ok(())
    }

    async fn handle_configure_tracker_path_to_card(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        (repository_id, space_id, qa_column_id, extract_pattern): (i32, i32, i32, String),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path_to_card = match msg.text() {
            Some(t) => t.trim().to_string(),
            None => {
                bot.send_message(msg.chat.id, "❌ Введите текстовое значение.").await?;
                return Ok(());
            }
        };

        let cmd = CreateRepositoryTaskTrackerCommand {
            repository_id: RepositoryId(repository_id),
            space_id,
            qa_column_id,
            extract_pattern_regexp: extract_pattern,
            path_to_card,
        };

        match executors
            .commands
            .create_repository_task_tracker
            .execute(&cmd)
            .await
        {
            Ok(response) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "✅ Таск-трекер для репозитория ID <b>{}</b> успешно настроен.",
                        response.tracker.repository_id.0
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create repository task tracker");
                bot.send_message(msg.chat.id, format!("❌ Ошибка настройки таск-трекера: {e}"))
                    .await?;
            }
        }

        dialogue.exit().await.ok();

        Ok(())
    }
}
