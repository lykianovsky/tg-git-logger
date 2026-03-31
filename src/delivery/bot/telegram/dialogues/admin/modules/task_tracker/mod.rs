use crate::application::repository::commands::update_repository_task_tracker::command::UpdateRepositoryTaskTrackerCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::TelegramBotDialogueAdminState;
use crate::delivery::bot::telegram::dialogues::admin::helpers::{db_error_message, extract_text};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::admin_task_tracker::TelegramBotAdminTaskTrackerAction;
use crate::delivery::bot::telegram::keyboards::actions::admin_task_tracker_edit_field::TelegramBotAdminTaskTrackerEditField;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::task::ports::task_tracker_client::TaskTrackerClient;
use crate::utils::builder::message::MessageBuilder;
use std::error::Error;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub struct TelegramBotDialogueAdminTaskTrackerDispatcher {}

impl TelegramBotDialogueAdminTaskTrackerDispatcher {
    /// Ветка выбора репозитория (первый шаг).
    pub fn query_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry().branch(
            case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectRepository]
                .endpoint(Self::handle_select_repository),
        )
    }

    /// Ветки callback для меню, выбора поля и интерактивного выбора через API.
    pub fn menu_query_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerMenu { repository_id }]
                    .endpoint(Self::handle_menu),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerEditSelectField {
                    repository_id
                }]
                .endpoint(Self::handle_edit_select_field),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectSpace {
                    repository_id
                }]
                .endpoint(Self::handle_select_space),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectBoard {
                    repository_id,
                    space_id
                }]
                .endpoint(Self::handle_select_board),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectColumn {
                    repository_id,
                    space_id,
                    board_id
                }]
                .endpoint(Self::handle_select_column),
            )
    }

    /// Ветки для текстовых сообщений (ввод паттерна).
    pub fn message_branches()
    -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
        dptree::entry()
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerEnterPattern {
                    repository_id,
                    space_id,
                    qa_column_id
                }]
                .endpoint(Self::handle_enter_pattern),
            )
            .branch(
                case![TelegramBotDialogueAdminState::ConfigureTaskTrackerEditExtractPattern {
                    repository_id
                }]
                .endpoint(Self::handle_edit_extract_pattern),
            )
    }

    // ── Выбор репозитория ────────────────────────────────────────────────────

    async fn handle_select_repository(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        task_tracker_client: Arc<dyn TaskTrackerClient>,
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

        let existing = executors
            .commands
            .update_repository_task_tracker
            .task_tracker_repo
            .find_by_repository_id(RepositoryId(repository_id))
            .await;

        match existing {
            Ok(_) => {
                let keyboard = KeyboardBuilder::new()
                    .row::<TelegramBotAdminTaskTrackerAction>(vec![
                        TelegramBotAdminTaskTrackerAction::View,
                        TelegramBotAdminTaskTrackerAction::Edit,
                    ])
                    .row::<TelegramBotAdminTaskTrackerAction>(vec![
                        TelegramBotAdminTaskTrackerAction::Reconfigure,
                    ])
                    .build();

                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ConfigureTaskTrackerMenu { repository_id },
                    ))
                    .await?;

                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    "⚙️ Настройки таск-трекера уже заданы. Что хотите сделать?",
                )
                .reply_markup(keyboard)
                .await?;
            }
            Err(_) => {
                Self::start_space_selection(
                    &bot,
                    &dialogue,
                    &task_tracker_client,
                    msg.chat().id,
                    msg.id(),
                    repository_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    // ── Меню существующих настроек ───────────────────────────────────────────

    async fn handle_menu(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        task_tracker_client: Arc<dyn TaskTrackerClient>,
        query: CallbackQuery,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let action = match TelegramBotAdminTaskTrackerAction::from_callback_data(data) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!(error = %e, "Unknown task tracker menu action");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        match action {
            TelegramBotAdminTaskTrackerAction::View => {
                let tracker = executors
                    .commands
                    .update_repository_task_tracker
                    .task_tracker_repo
                    .find_by_repository_id(RepositoryId(repository_id))
                    .await;

                match tracker {
                    Ok(t) => {
                        let text = MessageBuilder::new()
                            .with_html_escape(true)
                            .bold("📋 Настройки таск-трекера")
                            .empty_line()
                            .section_code("🏢 Space ID", &t.space_id.to_string())
                            .section_code("📋 QA Column ID", &t.qa_column_id.to_string())
                            .section_code("🔍 Regex паттерн", &t.extract_pattern_regexp)
                            .section_code("🗂 Путь к карточке", &t.path_to_card)
                            .build();

                        bot.edit_message_text(msg.chat().id, msg.id(), text)
                            .parse_mode(ParseMode::Html)
                            .reply_markup(InlineKeyboardMarkup::default())
                            .await?;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to load task tracker settings");
                        bot.edit_message_text(
                            msg.chat().id,
                            msg.id(),
                            db_error_message("загрузить настройки таск-трекера"),
                        )
                        .await?;
                    }
                }
                dialogue.exit().await.ok();
            }
            TelegramBotAdminTaskTrackerAction::Edit => {
                let keyboard = KeyboardBuilder::new()
                    .row::<TelegramBotAdminTaskTrackerEditField>(vec![
                        TelegramBotAdminTaskTrackerEditField::ExtractPattern,
                        TelegramBotAdminTaskTrackerEditField::Reconfigure,
                    ])
                    .build();

                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ConfigureTaskTrackerEditSelectField {
                            repository_id,
                        },
                    ))
                    .await?;

                bot.edit_message_text(msg.chat().id, msg.id(), "✏️ Что хотите изменить?")
                    .reply_markup(keyboard)
                    .await?;
            }
            TelegramBotAdminTaskTrackerAction::Reconfigure => {
                Self::start_space_selection(
                    &bot,
                    &dialogue,
                    &task_tracker_client,
                    msg.chat().id,
                    msg.id(),
                    repository_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    // ── Редактирование поля ──────────────────────────────────────────────────

    async fn handle_edit_select_field(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        task_tracker_client: Arc<dyn TaskTrackerClient>,
        query: CallbackQuery,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let field = match TelegramBotAdminTaskTrackerEditField::from_callback_data(data) {
            Ok(f) => f,
            Err(e) => {
                tracing::error!(error = %e, "Unknown task tracker edit field");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        match field {
            TelegramBotAdminTaskTrackerEditField::ExtractPattern => {
                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ConfigureTaskTrackerEditExtractPattern {
                            repository_id,
                        },
                    ))
                    .await?;

                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    "🔍 Введите новый regex-паттерн:\n\nВводите как есть, без экранирования. Например: \\bZB-(\\d+)\\b",
                )
                .reply_markup(InlineKeyboardMarkup::default())
                .await?;
            }
            TelegramBotAdminTaskTrackerEditField::Reconfigure => {
                Self::start_space_selection(
                    &bot,
                    &dialogue,
                    &task_tracker_client,
                    msg.chat().id,
                    msg.id(),
                    repository_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_edit_extract_pattern(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let new_pattern = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(msg.chat.id, "❌ Введите regex-паттерн текстом.")
                    .await?;
                return Ok(());
            }
        };

        let mut tracker = match executors
            .commands
            .update_repository_task_tracker
            .task_tracker_repo
            .find_by_repository_id(RepositoryId(repository_id))
            .await
        {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(error = %e, "Failed to load task tracker for patch");
                bot.send_message(
                    msg.chat.id,
                    db_error_message("загрузить настройки таск-трекера"),
                )
                .await?;
                dialogue.exit().await.ok();
                return Ok(());
            }
        };

        tracker.extract_pattern_regexp = new_pattern;

        let cmd = UpdateRepositoryTaskTrackerCommand {
            repository_id: RepositoryId(repository_id),
            space_id: tracker.space_id,
            qa_column_id: tracker.qa_column_id,
            extract_pattern_regexp: tracker.extract_pattern_regexp,
            path_to_card: tracker.path_to_card,
        };

        let loading = bot
            .send_message(msg.chat.id, "⏳ Сохраняем изменения...")
            .await?;

        match executors
            .commands
            .update_repository_task_tracker
            .execute(&cmd)
            .await
        {
            Ok(_) => {
                bot.edit_message_text(msg.chat.id, loading.id, "✅ Regex паттерн обновлён.")
                    .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to update extract pattern");
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    db_error_message("сохранить изменения"),
                )
                .await?;
            }
        }

        dialogue.exit().await.ok();
        Ok(())
    }

    // ── Интерактивный выбор через API ────────────────────────────────────────

    async fn start_space_selection(
        bot: &Bot,
        dialogue: &TelegramBotDialogueType,
        task_tracker_client: &Arc<dyn TaskTrackerClient>,
        chat_id: ChatId,
        message_id: teloxide::types::MessageId,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let loading_msg = bot
            .edit_message_text(chat_id, message_id, "⏳ Загружаю список пространств...")
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;

        let spaces = match task_tracker_client.list_spaces().await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "Failed to load spaces from task tracker");
                bot.edit_message_text(
                    chat_id,
                    loading_msg.id,
                    "❌ Не удалось загрузить список пространств. Проверьте подключение к Kaiten.",
                )
                .await?;
                dialogue.exit().await.ok();
                return Ok(());
            }
        };

        if spaces.is_empty() {
            bot.edit_message_text(chat_id, loading_msg.id, "❌ Пространства не найдены.")
                .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }

        let buttons: Vec<Vec<InlineKeyboardButton>> = spaces
            .into_iter()
            .map(|s| vec![InlineKeyboardButton::callback(s.title, s.id.to_string())])
            .collect();

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectSpace { repository_id },
            ))
            .await?;

        bot.edit_message_text(chat_id, loading_msg.id, "🏢 Выберите пространство (space):")
            .reply_markup(InlineKeyboardMarkup::new(buttons))
            .await?;

        Ok(())
    }

    async fn handle_select_space(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        task_tracker_client: Arc<dyn TaskTrackerClient>,
        query: CallbackQuery,
        repository_id: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let space_id: i32 = match data.parse() {
            Ok(v) => v,
            Err(_) => {
                tracing::error!(data = %data, "Invalid space_id in callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let loading = bot
            .edit_message_text(msg.chat().id, msg.id(), "⏳ Загружаю список досок...")
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;

        let boards = match task_tracker_client.list_boards(space_id).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!(error = %e, space_id = space_id, "Failed to load boards");
                bot.edit_message_text(
                    msg.chat().id,
                    loading.id,
                    "❌ Не удалось загрузить список досок.",
                )
                .await?;
                dialogue.exit().await.ok();
                return Ok(());
            }
        };

        if boards.is_empty() {
            bot.edit_message_text(
                msg.chat().id,
                loading.id,
                "❌ В выбранном пространстве нет досок.",
            )
            .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }

        let buttons: Vec<Vec<InlineKeyboardButton>> = boards
            .into_iter()
            .map(|b| vec![InlineKeyboardButton::callback(b.title, b.id.to_string())])
            .collect();

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectBoard {
                    repository_id,
                    space_id,
                },
            ))
            .await?;

        bot.edit_message_text(msg.chat().id, loading.id, "📋 Выберите доску:")
            .reply_markup(InlineKeyboardMarkup::new(buttons))
            .await?;

        Ok(())
    }

    async fn handle_select_board(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        task_tracker_client: Arc<dyn TaskTrackerClient>,
        query: CallbackQuery,
        (repository_id, space_id): (i32, i32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let board_id: i32 = match data.parse() {
            Ok(v) => v,
            Err(_) => {
                tracing::error!(data = %data, "Invalid board_id in callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        let loading = bot
            .edit_message_text(msg.chat().id, msg.id(), "⏳ Загружаю список колонок...")
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;

        let columns = match task_tracker_client.list_columns(board_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = %e, board_id = board_id, "Failed to load columns");
                bot.edit_message_text(
                    msg.chat().id,
                    loading.id,
                    "❌ Не удалось загрузить список колонок.",
                )
                .await?;
                dialogue.exit().await.ok();
                return Ok(());
            }
        };

        if columns.is_empty() {
            bot.edit_message_text(
                msg.chat().id,
                loading.id,
                "❌ На выбранной доске нет колонок.",
            )
            .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }

        let buttons: Vec<Vec<InlineKeyboardButton>> = columns
            .into_iter()
            .map(|c| vec![InlineKeyboardButton::callback(c.title, c.id.to_string())])
            .collect();

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerSelectColumn {
                    repository_id,
                    space_id,
                    board_id,
                },
            ))
            .await?;

        bot.edit_message_text(
            msg.chat().id,
            loading.id,
            "🎯 Выберите колонку QA (куда перемещать задачи):",
        )
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .await?;

        Ok(())
    }

    async fn handle_select_column(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        query: CallbackQuery,
        (repository_id, space_id, _board_id): (i32, i32, i32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id.clone()).await?;

        let data = query.data.as_deref().unwrap_or("");
        let qa_column_id: i32 = match data.parse() {
            Ok(v) => v,
            Err(_) => {
                tracing::error!(data = %data, "Invalid column_id in callback");
                return Ok(());
            }
        };

        let msg = match query.message {
            Some(m) => m,
            None => return Ok(()),
        };

        dialogue
            .update(TelegramBotDialogueState::Admin(
                TelegramBotDialogueAdminState::ConfigureTaskTrackerEnterPattern {
                    repository_id,
                    space_id,
                    qa_column_id,
                },
            ))
            .await?;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            "🔍 Введите regex-паттерн для извлечения ID задачи из PR:\n\nВводите как есть, без экранирования. Например: \\bZB-(\\d+)\\b",
        )
        .reply_markup(InlineKeyboardMarkup::default())
        .await?;

        Ok(())
    }

    // ── Ввод паттерна и финальное сохранение ─────────────────────────────────

    async fn handle_enter_pattern(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        msg: Message,
        (repository_id, space_id, qa_column_id): (i32, i32, i32),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let extract_pattern = match extract_text(&msg) {
            Some(v) => v,
            None => {
                bot.send_message(msg.chat.id, "❌ Введите regex-паттерн текстом.")
                    .await?;
                return Ok(());
            }
        };

        let path_to_card = format!("/space/{}/boards/card/{{id}}", space_id);

        let cmd = UpdateRepositoryTaskTrackerCommand {
            repository_id: RepositoryId(repository_id),
            space_id,
            qa_column_id,
            extract_pattern_regexp: extract_pattern,
            path_to_card,
        };

        let loading = bot
            .send_message(msg.chat.id, "⏳ Сохраняем настройки...")
            .await?;

        match executors
            .commands
            .update_repository_task_tracker
            .execute(&cmd)
            .await
        {
            Ok(_) => {
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    "✅ Настройки таск-трекера успешно сохранены.",
                )
                .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to save task tracker settings");
                bot.edit_message_text(
                    msg.chat.id,
                    loading.id,
                    db_error_message("сохранить настройки таск-трекера"),
                )
                .await?;
            }
        }

        dialogue.exit().await.ok();
        Ok(())
    }
}
