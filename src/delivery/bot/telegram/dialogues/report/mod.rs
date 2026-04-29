use crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery;
use crate::application::version_control::queries::build_report::command::{
    BuildVersionControlDateRangeReportExecutorCommand,
    BuildVersionControlDateRangeReportExecutorCommandForWho,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::date_range::TelegramBotDateRangeAction;
use crate::delivery::bot::telegram::keyboards::actions::for_who::TelegramBotForWhoAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::shared::date::range::DateRange;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use chrono::NaiveDate;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt};
use teloxide::dptree::{Handler, case};
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::{Requester, Update};
use teloxide::types::ChatId;
use teloxide::types::{
    CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode,
};
use teloxide::{Bot, dptree};

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

    EnterDateSince {
        for_who_action: TelegramBotForWhoAction,
        repository_id: i32,
        branch: String,
    },

    EnterDateUntil {
        for_who_action: TelegramBotForWhoAction,
        repository_id: i32,
        branch: String,
        since: String,
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

        let messages = Update::filter_message()
            .branch(
                case![TelegramBotDialogueReportByDateRangeState::EnterBranch {
                    for_who_action,
                    repository_id
                }]
                .endpoint(enter_branch),
            )
            .branch(
                case![TelegramBotDialogueReportByDateRangeState::EnterDateSince {
                    for_who_action,
                    repository_id,
                    branch
                }]
                .endpoint(enter_date_since),
            )
            .branch(
                case![TelegramBotDialogueReportByDateRangeState::EnterDateUntil {
                    for_who_action,
                    repository_id,
                    branch,
                    since
                }]
                .endpoint(enter_date_until),
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
                t!("telegram_bot.dialogues.report.load_repos_error").to_string(),
            )
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;
            dialogue.exit().await.ok();
        }
        Ok(response) if response.repositories.is_empty() => {
            bot.edit_message_text(
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.report.no_bound_repos").to_string(),
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

            bot.edit_message_text(
                chat_id,
                message_id,
                t!("telegram_bot.dialogues.report.select_repository").to_string(),
            )
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
        t!("telegram_bot.dialogues.report.enter_branch").to_string(),
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
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.report.branch_required").to_string(),
            )
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
        .row::<TelegramBotDateRangeAction>(vec![TelegramBotDateRangeAction::Custom])
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

    bot.send_message(
        msg.chat.id,
        t!("telegram_bot.dialogues.report.select_date_range").to_string(),
    )
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

    let msg = query.message.unwrap();
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    if matches!(date_range_action, TelegramBotDateRangeAction::Custom) {
        dialogue
            .update(TelegramBotDialogueState::ReportByDateRange(
                TelegramBotDialogueReportByDateRangeState::EnterDateSince {
                    for_who_action,
                    repository_id,
                    branch,
                },
            ))
            .await?;

        bot.edit_message_text(
            chat_id,
            message_id,
            t!("telegram_bot.dialogues.report.enter_date_since").to_string(),
        )
        .reply_markup(InlineKeyboardMarkup::default())
        .await?;

        return Ok(());
    }

    let date_range = match date_range_action {
        TelegramBotDateRangeAction::LastWeek => DateRange::last_week(),
        TelegramBotDateRangeAction::Last2Weeks => DateRange::last_2_weeks(),
        TelegramBotDateRangeAction::LastMonth => DateRange::last_month(),
        TelegramBotDateRangeAction::ThisMonth => DateRange::this_month(),
        TelegramBotDateRangeAction::Custom => unreachable!(),
    };

    bot.edit_message_text(
        chat_id,
        message_id,
        t!("telegram_bot.dialogues.report.loading").to_string(),
    )
    .reply_markup(InlineKeyboardMarkup::default())
    .await?;

    let cmd = BuildVersionControlDateRangeReportExecutorCommand {
        social_user_id: SocialUserId(query.from.id.0 as i32),
        date_range,
        for_who: map_for_who(&for_who_action),
        repository_id: RepositoryId(repository_id),
        branch,
    };

    execute_and_send_report(&bot, &dialogue, &executors, chat_id, message_id, cmd).await
}

fn map_for_who(
    action: &TelegramBotForWhoAction,
) -> BuildVersionControlDateRangeReportExecutorCommandForWho {
    match action {
        TelegramBotForWhoAction::Me => BuildVersionControlDateRangeReportExecutorCommandForWho::Me,
        TelegramBotForWhoAction::Repository => {
            BuildVersionControlDateRangeReportExecutorCommandForWho::Repository
        }
    }
}

async fn execute_and_send_report(
    bot: &Bot,
    dialogue: &TelegramBotDialogueType,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    loading_message_id: teloxide::types::MessageId,
    cmd: BuildVersionControlDateRangeReportExecutorCommand,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let executor = executors.queries.build_report_by_range.clone();

    match executor.execute(&cmd).await {
        Ok(response) => {
            let text = MessageBuilder::new()
                .line(t!("telegram_bot.dialogues.report.ready").as_ref())
                .empty_line()
                .link(
                    t!("telegram_bot.dialogues.report.open_link").as_ref(),
                    &response.report_url,
                )
                .build();

            bot.edit_message_text(chat_id, loading_message_id, text)
                .parse_mode(ParseMode::Html)
                .await?;

            dialogue.exit().await?;
        }
        Err(error) => {
            let error_text = executor.friendly_error_message(&error);

            bot.edit_message_text(
                chat_id,
                loading_message_id,
                format!(
                    "{}\n\n{}",
                    error_text,
                    t!("telegram_bot.dialogues.report.try_another_range")
                ),
            )
            .await?;

            dialogue.exit().await.ok();
        }
    }

    Ok(())
}

async fn enter_date_since(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (for_who_action, repository_id, branch): (TelegramBotForWhoAction, i32, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input = match msg
        .text()
        .map(|t| t.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(v) => v,
        None => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.report.date_required").to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    if NaiveDate::parse_from_str(&input, "%d.%m.%Y").is_err() {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.report.date_format_error_since").to_string(),
        )
        .await?;
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReportByDateRange(
            TelegramBotDialogueReportByDateRangeState::EnterDateUntil {
                for_who_action,
                repository_id,
                branch,
                since: input,
            },
        ))
        .await?;

    bot.send_message(
        msg.chat.id,
        t!("telegram_bot.dialogues.report.enter_date_until").to_string(),
    )
    .await?;

    Ok(())
}

async fn enter_date_until(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    (for_who_action, repository_id, branch, since): (TelegramBotForWhoAction, i32, String, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let until_input = match msg
        .text()
        .map(|t| t.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(v) => v,
        None => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.report.date_required").to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    let since_date = match NaiveDate::parse_from_str(&since, "%d.%m.%Y") {
        Ok(d) => d,
        Err(_) => {
            tracing::error!(since = %since, "Failed to re-parse since date in enter_date_until");
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.report.internal_error").to_string(),
            )
            .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let until_date = match NaiveDate::parse_from_str(&until_input, "%d.%m.%Y") {
        Ok(d) => d,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.report.date_format_error_until").to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    if until_date < since_date {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.report.date_range_invalid").to_string(),
        )
        .await?;
        return Ok(());
    }

    let since_dt = since_date
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always valid")
        .and_utc();

    let until_dt = until_date
        .and_hms_opt(23, 59, 59)
        .expect("23:59:59 is always valid")
        .and_utc();

    let date_range = DateRange::new(since_dt, until_dt);

    let social_user_id = SocialUserId(msg.from.as_ref().map(|u| u.id.0 as i32).unwrap_or_default());

    let loading = bot
        .send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.report.loading").to_string(),
        )
        .await?;

    let cmd = BuildVersionControlDateRangeReportExecutorCommand {
        social_user_id,
        date_range,
        for_who: map_for_who(&for_who_action),
        repository_id: RepositoryId(repository_id),
        branch,
    };

    execute_and_send_report(&bot, &dialogue, &executors, msg.chat.id, loading.id, cmd).await
}
