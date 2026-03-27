use crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery;
use crate::application::version_control::queries::build_report::command::{
    BuildVersionControlDateRangeReportExecutorCommand,
    BuildVersionControlDateRangeReportExecutorCommandForWho,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::date_range::TelegramBotDateRangeAction;
use crate::delivery::bot::telegram::keyboards::actions::for_who::TelegramBotForWhoAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::shared::date::range::DateRange;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::{Requester, Update};
use teloxide::types::{
    CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode,
};
use teloxide::{dptree, Bot};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotDialogueReportByDateRangeState {
    #[default]
    For,

    SelectRepository {
        for_who_action: TelegramBotForWhoAction,
    },

    EnterBranch {
        for_who_action: TelegramBotForWhoAction,
        repository_id: i32,
    },

    DateRange {
        for_who_action: TelegramBotForWhoAction,
        repository_id: i32,
        branch: String,
    },
}

pub struct TelegramBotDialogueReportByDateRangeDispatcher {}

impl TelegramBotDialogueReportByDateRangeDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let callbacks = Update::filter_callback_query()
            .branch(case![TelegramBotDialogueReportByDateRangeState::For].endpoint(choose_for_who))
            .branch(
                case![
                    TelegramBotDialogueReportByDateRangeState::SelectRepository { for_who_action }
                ]
                .endpoint(choose_repository),
            )
            .branch(
                case![TelegramBotDialogueReportByDateRangeState::DateRange {
                    for_who_action,
                    repository_id,
                    branch
                }]
                .endpoint(create_report_by_date_range),
            );

        let messages = Update::filter_message().branch(
            case![TelegramBotDialogueReportByDateRangeState::EnterBranch {
                for_who_action,
                repository_id
            }]
            .endpoint(enter_branch),
        );

        dptree::entry().branch(callbacks).branch(messages)
    }
}

async fn choose_for_who(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let callback_data = query.data.as_deref().unwrap_or("");
    let for_who_action = match TelegramBotForWhoAction::from_callback_data(callback_data) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Unknown for_who action");
            return Ok(());
        }
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);
    let msg = query.message.unwrap();
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    let bound_repos_response = executors
        .queries
        .get_user_bound_repositories
        .execute(&GetUserBoundRepositoriesQuery { social_user_id })
        .await;

    match bound_repos_response {
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user bound repositories");
            bot.edit_message_text(
                chat_id,
                message_id,
                "❌ Не удалось загрузить список репозиториев. Попробуйте позже.",
            )
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;
            dialogue.exit().await.ok();
        }
        Ok(response) if response.repositories.is_empty() => {
            bot.edit_message_text(
                chat_id,
                message_id,
                "У вас нет привязанных репозиториев. Используйте /bind_repository.",
            )
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;
            dialogue.exit().await.ok();
        }
        Ok(response) => {
            let rows: Vec<Vec<InlineKeyboardButton>> = response
                .repositories
                .iter()
                .map(|r| {
                    vec![InlineKeyboardButton::callback(
                        format!("{}/{}", r.owner, r.name),
                        r.id.0.to_string(),
                    )]
                })
                .collect();

            let keyboard = InlineKeyboardMarkup::new(rows);

            dialogue
                .update(TelegramBotDialogueState::ReportByDateRange(
                    TelegramBotDialogueReportByDateRangeState::SelectRepository { for_who_action },
                ))
                .await?;

            bot.edit_message_text(chat_id, message_id, "📦 Выберите репозиторий:")
                .reply_markup(keyboard)
                .await?;
        }
    }

    Ok(())
}

async fn choose_repository(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    query: CallbackQuery,
    for_who_action: TelegramBotForWhoAction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");
    let repository_id: i32 = match data.parse() {
        Ok(v) => v,
        Err(_) => {
            tracing::error!(data = %data, "Invalid repository_id in report callback");
            return Ok(());
        }
    };

    let msg = query.message.unwrap();
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    dialogue
        .update(TelegramBotDialogueState::ReportByDateRange(
            TelegramBotDialogueReportByDateRangeState::EnterBranch {
                for_who_action,
                repository_id,
            },
        ))
        .await?;

    bot.edit_message_text(
        chat_id,
        message_id,
        "🌿 Введите название ветки (например: main, dev, master):",
    )
    .reply_markup(InlineKeyboardMarkup::default())
    .await?;

    Ok(())
}

async fn enter_branch(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (for_who_action, repository_id): (TelegramBotForWhoAction, i32),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let branch = match msg
        .text()
        .map(|t| t.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(b) => b,
        None => {
            bot.send_message(msg.chat.id, "❌ Введите название ветки текстом.")
                .await?;
            return Ok(());
        }
    };

    let keyboard = KeyboardBuilder::new()
        .row::<TelegramBotDateRangeAction>(vec![
            TelegramBotDateRangeAction::LastWeek,
            TelegramBotDateRangeAction::Last2Weeks,
        ])
        .row::<TelegramBotDateRangeAction>(vec![
            TelegramBotDateRangeAction::LastMonth,
            TelegramBotDateRangeAction::ThisMonth,
        ])
        .build();

    dialogue
        .update(TelegramBotDialogueState::ReportByDateRange(
            TelegramBotDialogueReportByDateRangeState::DateRange {
                for_who_action,
                repository_id,
                branch,
            },
        ))
        .await?;

    bot.send_message(msg.chat.id, "📊 Выберите диапазон дат:")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn create_report_by_date_range(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    (for_who_action, repository_id, branch): (TelegramBotForWhoAction, i32, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let callback_data = query.data.as_deref().unwrap_or("");

    let date_range_action = match TelegramBotDateRangeAction::from_callback_data(callback_data) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Unknown date_range action");
            return Ok(());
        }
    };

    let date_range = match date_range_action {
        TelegramBotDateRangeAction::LastWeek => DateRange::last_week(),
        TelegramBotDateRangeAction::Last2Weeks => DateRange::last_2_weeks(),
        TelegramBotDateRangeAction::LastMonth => DateRange::last_month(),
        TelegramBotDateRangeAction::ThisMonth => DateRange::this_month(),
    };

    let for_who = match for_who_action {
        TelegramBotForWhoAction::Me => BuildVersionControlDateRangeReportExecutorCommandForWho::Me,
        TelegramBotForWhoAction::Repository => {
            BuildVersionControlDateRangeReportExecutorCommandForWho::Repository
        }
    };

    let msg = query.message.unwrap();
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    bot.edit_message_text(chat_id, message_id, "⏳ Загружаем отчёт...")
        .reply_markup(InlineKeyboardMarkup::default())
        .await?;

    let cmd = BuildVersionControlDateRangeReportExecutorCommand {
        social_user_id: SocialUserId(query.from.id.0 as i32),
        date_range,
        for_who,
        repository_id: RepositoryId(repository_id),
        branch,
    };

    let executor = executors.queries.build_report_by_range.clone();

    match executor.execute(&cmd).await {
        Ok(response) => {
            let msg = MessageBuilder::new()
                .line("✅ Отчёт готов!")
                .empty_line()
                .link("📊 Открыть полный отчёт", &response.report_url)
                .build();

            tracing::debug!("REPORT URL: {}", response.report_url);

            bot.edit_message_text(chat_id, message_id, msg)
                .parse_mode(ParseMode::Html)
                .await?;

            dialogue.exit().await?;
        }
        Err(error) => {
            let error_text = executor.friendly_error_message(&error);

            // Не завершаем диалог — оставляем состояние DateRange для повтора
            let retry_keyboard =
                InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                    "🔄 Попробовать снова",
                    date_range_action.to_callback_data(),
                )]]);

            bot.edit_message_text(
                chat_id,
                message_id,
                format!(
                    "{}\n\nМожете попробовать снова с теми же параметрами.",
                    error_text
                ),
            )
            .reply_markup(retry_keyboard)
            .await?;
        }
    };

    Ok(())
}
