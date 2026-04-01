pub mod helpers;
pub mod modules;

use crate::application::monitoring::queries::get_queues_stats::query::GetQueuesStatsQuery;
use crate::domain::shared::command::CommandExecutor as _;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::admin::modules::health_ping::TelegramBotDialogueAdminHealthPingDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::repository::TelegramBotDialogueAdminRepositoryDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::task_tracker::TelegramBotDialogueAdminTaskTrackerDispatcher;
use crate::delivery::bot::telegram::dialogues::admin::modules::users::TelegramBotDialogueAdminUsersDispatcher;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::dialogues::helpers::parse_callback;
use crate::delivery::bot::telegram::keyboards::actions::admin::TelegramBotAdminAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::utils::builder::message::MessageBuilder;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::types::ParseMode;
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

    // Просмотр
    ViewRepositorySelect,

    // Удаление
    DeleteRepositorySelect,
    DeleteRepositoryConfirm {
        repository_id: i32,
    },

    // ── Таск-трекер ─────────────────────────────────────────────────────────
    ConfigureTaskTrackerSelectRepository,

    // Меню существующих настроек (View / Edit / Reconfigure)
    ConfigureTaskTrackerMenu {
        repository_id: i32,
    },

    // Редактирование паттерна (единственное поле, которое нельзя получить из API)
    ConfigureTaskTrackerEditSelectField {
        repository_id: i32,
    },
    ConfigureTaskTrackerEditExtractPattern {
        repository_id: i32,
    },

    // Интерактивный выбор через API (создание / перенастройка)
    ConfigureTaskTrackerSelectSpace {
        repository_id: i32,
    },
    ConfigureTaskTrackerSelectBoard {
        repository_id: i32,
        space_id: i32,
    },
    ConfigureTaskTrackerSelectColumn {
        repository_id: i32,
        space_id: i32,
        board_id: i32,
    },
    ConfigureTaskTrackerEnterPattern {
        repository_id: i32,
        space_id: i32,
        qa_column_id: i32,
    },

    // ── Пинги ──────────────────────────────────────────────────────────────
    HealthPingList,

    HealthPingCreateName,
    HealthPingCreateUrl {
        name: String,
    },
    HealthPingCreateInterval {
        name: String,
        url: String,
    },

    HealthPingEditSelect,
    HealthPingEditMenu {
        ping_id: i32,
    },
    HealthPingEditName {
        ping_id: i32,
    },
    HealthPingEditUrl {
        ping_id: i32,
    },
    HealthPingEditInterval {
        ping_id: i32,
    },

    HealthPingDeleteConfirm {
        ping_id: i32,
    },

    // ── Пользователи ──────────────────────────────────────────────────────
    UserList,

    UserMenu {
        user_id: i32,
    },

    UserAssignRole {
        user_id: i32,
    },

    UserRemoveRole {
        user_id: i32,
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
            .branch(TelegramBotDialogueAdminTaskTrackerDispatcher::query_branches())
            .branch(TelegramBotDialogueAdminTaskTrackerDispatcher::menu_query_branches())
            .branch(TelegramBotDialogueAdminHealthPingDispatcher::query_branches())
            .branch(TelegramBotDialogueAdminUsersDispatcher::query_branches());

        let messages = Update::filter_message()
            .branch(TelegramBotDialogueAdminRepositoryDispatcher::message_branches())
            .branch(TelegramBotDialogueAdminTaskTrackerDispatcher::message_branches())
            .branch(TelegramBotDialogueAdminHealthPingDispatcher::message_branches());

        dptree::entry().branch(callback_queries).branch(messages)
    }

    async fn handle_menu(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = match parse_callback::<TelegramBotAdminAction>(&bot, &query)
            .await?
        {
            Some(c) => c,
            None => return Ok(()),
        };

        let chat_id = ctx.chat_id;
        let message_id = ctx.message_id;

        match ctx.action {
            TelegramBotAdminAction::ConfigureRepository => {
                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::ConfigureRepository,
                    ))
                    .await?;

                let keyboard = KeyboardBuilder::new()
                    .row::<TelegramBotAdminRepositoryAction>(vec![
                        TelegramBotAdminRepositoryAction::View,
                    ])
                    .row::<TelegramBotAdminRepositoryAction>(vec![
                        TelegramBotAdminRepositoryAction::Create,
                        TelegramBotAdminRepositoryAction::Edit,
                    ])
                    .row::<TelegramBotAdminRepositoryAction>(vec![
                        TelegramBotAdminRepositoryAction::Delete,
                    ])
                    .build();

                bot.edit_message_text(
                    chat_id,
                    message_id,
                    t!("telegram_bot.dialogues.admin.repositories_title").to_string(),
                )
                    .reply_markup(keyboard)
                    .await?;
            }
            TelegramBotAdminAction::QueuesStats => {
                let response = match executors
                    .queries
                    .get_queues_stats
                    .execute(&GetQueuesStatsQuery)
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to get queues stats");
                        bot.edit_message_text(
                            chat_id,
                            message_id,
                            t!("telegram_bot.dialogues.admin.queues_stats_error").to_string(),
                        )
                            .await?;
                        dialogue.exit().await.ok();
                        return Ok(());
                    }
                };

                let mut builder = MessageBuilder::new()
                    .bold(t!("telegram_bot.dialogues.admin.queues_stats_title").as_ref())
                    .empty_line();

                for stat in &response.stats {
                    let status = if stat.pending_messages == 0 {
                        t!("telegram_bot.dialogues.admin.queue_status.idle")
                    } else if stat.pending_messages < 10 {
                        t!("telegram_bot.dialogues.admin.queue_status.active")
                    } else {
                        t!("telegram_bot.dialogues.admin.queue_status.overloaded")
                    };

                    builder = builder
                        .section_bold(&stat.queue_name, status.as_ref())
                        .section_code(
                            t!("telegram_bot.dialogues.admin.queue_workers").as_ref(),
                            &stat.active_workers.to_string(),
                        )
                        .section_code(
                            t!("telegram_bot.dialogues.admin.queue_pending").as_ref(),
                            &stat.pending_messages.to_string(),
                        )
                        .empty_line();
                }

                bot.edit_message_text(chat_id, message_id, builder.build())
                    .parse_mode(ParseMode::Html)
                    .await?;

                dialogue.exit().await.ok();
            }
            TelegramBotAdminAction::HealthPings => {
                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::HealthPingList,
                    ))
                    .await?;

                TelegramBotDialogueAdminHealthPingDispatcher::show_list(
                    &bot,
                    chat_id,
                    message_id,
                    &executors,
                )
                .await?;
            }

            TelegramBotAdminAction::ManageUsers => {
                dialogue
                    .update(TelegramBotDialogueState::Admin(
                        TelegramBotDialogueAdminState::UserList,
                    ))
                    .await?;

                TelegramBotDialogueAdminUsersDispatcher::show_list(
                    &bot,
                    chat_id,
                    message_id,
                    &executors,
                )
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
                        t!("telegram_bot.dialogues.admin.no_repositories").to_string(),
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
                    t!("telegram_bot.dialogues.admin.task_tracker_select_repository").to_string(),
                )
                .reply_markup(keyboard)
                .await?;
            }
        }

        Ok(())
    }
}
