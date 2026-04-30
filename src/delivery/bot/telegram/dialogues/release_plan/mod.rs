use crate::application::release_plan::commands::create_release_plan::command::CreateReleasePlanExecutorCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::dialogues::helpers::{close_menu, edit_menu};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::release_plan::{
    REPO_TOGGLE_PREFIX, TelegramBotReleasePlanCallSetupAction,
    TelegramBotReleasePlanOptionalAction, TelegramBotReleasePlanReposAction, repo_toggle_callback,
};
use crate::domain::release_plan::services::default_call_resolver::DefaultCallResolver;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Europe::Moscow;
use std::collections::HashSet;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotReleasePlanState {
    #[default]
    EnterDate,
    SelectRepositories {
        date_iso: String,
        selected: Vec<i32>,
    },
    ChooseCallSetup {
        date_iso: String,
        selected: Vec<i32>,
    },
    EnterCallDate {
        date_iso: String,
        selected: Vec<i32>,
    },
    EnterCallTime {
        date_iso: String,
        selected: Vec<i32>,
        call_date_iso: String,
    },
    EnterMeetingUrl {
        date_iso: String,
        selected: Vec<i32>,
        call_datetime_iso: String,
    },
    EnterNote {
        date_iso: String,
        selected: Vec<i32>,
        call_datetime_iso: String,
        meeting_url: Option<String>,
    },
}

pub struct TelegramBotReleasePlanDispatcher {}

impl TelegramBotReleasePlanDispatcher {
    pub fn new()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        let messages = Update::filter_message()
            .branch(case![TelegramBotReleasePlanState::EnterDate].endpoint(handle_date_input))
            .branch(
                case![TelegramBotReleasePlanState::EnterCallDate { date_iso, selected }]
                    .endpoint(handle_call_date_input),
            )
            .branch(
                case![TelegramBotReleasePlanState::EnterCallTime {
                    date_iso,
                    selected,
                    call_date_iso
                }]
                .endpoint(handle_call_time_input),
            )
            .branch(
                case![TelegramBotReleasePlanState::EnterMeetingUrl {
                    date_iso,
                    selected,
                    call_datetime_iso
                }]
                .endpoint(handle_meeting_url_input),
            )
            .branch(
                case![TelegramBotReleasePlanState::EnterNote {
                    date_iso,
                    selected,
                    call_datetime_iso,
                    meeting_url
                }]
                .endpoint(handle_note_input),
            );

        let queries = Update::filter_callback_query()
            .branch(
                case![TelegramBotReleasePlanState::SelectRepositories { date_iso, selected }]
                    .endpoint(handle_repos),
            )
            .branch(
                case![TelegramBotReleasePlanState::ChooseCallSetup { date_iso, selected }]
                    .endpoint(handle_call_setup),
            )
            .branch(
                case![TelegramBotReleasePlanState::EnterCallDate { date_iso, selected }]
                    .endpoint(handle_call_date_callback),
            )
            .branch(
                case![TelegramBotReleasePlanState::EnterCallTime {
                    date_iso,
                    selected,
                    call_date_iso
                }]
                .endpoint(handle_call_time_callback),
            )
            .branch(
                case![TelegramBotReleasePlanState::EnterMeetingUrl {
                    date_iso,
                    selected,
                    call_datetime_iso
                }]
                .endpoint(handle_meeting_url_callback),
            )
            .branch(
                case![TelegramBotReleasePlanState::EnterNote {
                    date_iso,
                    selected,
                    call_datetime_iso,
                    meeting_url
                }]
                .endpoint(handle_note_callback),
            );

        teloxide::dptree::entry().branch(messages).branch(queries)
    }
}

async fn handle_date_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or("").trim();
    let date = match NaiveDate::parse_from_str(text, "%d.%m.%Y") {
        Ok(d) => d,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.release_plan.invalid_date").to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::ReleasePlan(
            TelegramBotReleasePlanState::SelectRepositories {
                date_iso: date.format("%Y-%m-%d").to_string(),
                selected: Vec::new(),
            },
        ))
        .await?;

    send_repos_menu(&bot, msg.chat.id, &executors, &[]).await?;
    Ok(())
}

async fn build_repos_menu(
    executors: &Arc<ApplicationBoostrapExecutors>,
    selected: &[i32],
) -> InlineKeyboardMarkup {
    let all = executors
        .queries
        .get_all_repositories
        .execute(&crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery {})
        .await
        .map(|r| r.repositories)
        .unwrap_or_default();

    let selected_set: HashSet<i32> = selected.iter().copied().collect();

    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for repo in &all {
        let mark = if selected_set.contains(&repo.id.0) {
            "✅"
        } else {
            "⬜"
        };
        rows.push(vec![InlineKeyboardButton::callback(
            format!("{} {}/{}", mark, repo.owner, repo.name),
            repo_toggle_callback(repo.id.0),
        )]);
    }
    rows.push(vec![InlineKeyboardButton::callback(
        TelegramBotReleasePlanReposAction::SelectAll
            .label()
            .to_string(),
        TelegramBotReleasePlanReposAction::SelectAll
            .to_callback_data()
            .to_string(),
    )]);
    rows.push(vec![
        InlineKeyboardButton::callback(
            TelegramBotReleasePlanReposAction::Done.label().to_string(),
            TelegramBotReleasePlanReposAction::Done
                .to_callback_data()
                .to_string(),
        ),
        InlineKeyboardButton::callback(
            TelegramBotReleasePlanReposAction::Cancel
                .label()
                .to_string(),
            TelegramBotReleasePlanReposAction::Cancel
                .to_callback_data()
                .to_string(),
        ),
    ]);

    InlineKeyboardMarkup::new(rows)
}

async fn send_repos_menu(
    bot: &Bot,
    chat_id: ChatId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    selected: &[i32],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kb = build_repos_menu(executors, selected).await;
    bot.send_message(
        chat_id,
        t!("telegram_bot.dialogues.release_plan.select_repos").to_string(),
    )
    .reply_markup(kb)
    .await?;
    Ok(())
}

async fn edit_repos_menu(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    selected: &[i32],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kb = build_repos_menu(executors, selected).await;
    let text = t!("telegram_bot.dialogues.release_plan.select_repos").to_string();
    edit_menu(bot, chat_id, message_id, &text, Some(kb)).await
}

fn build_call_setup_menu(default_label: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            format!(
                "{} ({})",
                TelegramBotReleasePlanCallSetupAction::UseDefault.label(),
                default_label
            ),
            TelegramBotReleasePlanCallSetupAction::UseDefault
                .to_callback_data()
                .to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            TelegramBotReleasePlanCallSetupAction::EnterManually
                .label()
                .to_string(),
            TelegramBotReleasePlanCallSetupAction::EnterManually
                .to_callback_data()
                .to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            TelegramBotReleasePlanCallSetupAction::Cancel.label().to_string(),
            TelegramBotReleasePlanCallSetupAction::Cancel
                .to_callback_data()
                .to_string(),
        )],
    ])
}

fn build_optional_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            TelegramBotReleasePlanOptionalAction::Skip.label().to_string(),
            TelegramBotReleasePlanOptionalAction::Skip
                .to_callback_data()
                .to_string(),
        ),
        InlineKeyboardButton::callback(
            TelegramBotReleasePlanOptionalAction::Cancel
                .label()
                .to_string(),
            TelegramBotReleasePlanOptionalAction::Cancel
                .to_callback_data()
                .to_string(),
        ),
    ]])
}

fn build_cancel_only_menu(action: TelegramBotReleasePlanOptionalAction) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        action.label().to_string(),
        action.to_callback_data().to_string(),
    )]])
}

fn resolve_default_call(planned_date: NaiveDate, config: &ApplicationConfig) -> DateTime<Utc> {
    let resolver = DefaultCallResolver::new(
        config.release_plan.default_call_weekday,
        config.release_plan.default_call_time,
        Moscow,
    );
    resolver.resolve_for(planned_date)
}

fn format_default_call_label(call_dt_utc: DateTime<Utc>) -> String {
    call_dt_utc
        .with_timezone(&Moscow)
        .format("%d.%m %H:%M МСК")
        .to_string()
}

async fn handle_repos(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
    query: CallbackQuery,
    (date_iso, mut selected): (String, Vec<i32>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    if let Some(repo_id_str) = data.strip_prefix(REPO_TOGGLE_PREFIX) {
        if let Ok(repo_id) = repo_id_str.parse::<i32>() {
            if let Some(pos) = selected.iter().position(|x| *x == repo_id) {
                selected.remove(pos);
            } else {
                selected.push(repo_id);
            }
            dialogue
                .update(TelegramBotDialogueState::ReleasePlan(
                    TelegramBotReleasePlanState::SelectRepositories {
                        date_iso: date_iso.clone(),
                        selected: selected.clone(),
                    },
                ))
                .await?;
            edit_repos_menu(&bot, chat_id, message_id, &executors, &selected).await?;
        }
        return Ok(());
    }

    if let Ok(action) = TelegramBotReleasePlanReposAction::from_callback_data(data) {
        match action {
            TelegramBotReleasePlanReposAction::Cancel => {
                close_menu(&bot, chat_id, message_id).await;
                dialogue.exit().await.ok();
                return Ok(());
            }
            TelegramBotReleasePlanReposAction::SelectAll => {
                let all = executors
                    .queries
                    .get_all_repositories
                    .execute(&crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery {})
                    .await
                    .map(|r| r.repositories)
                    .unwrap_or_default();
                selected = all.iter().map(|r| r.id.0).collect();
                dialogue
                    .update(TelegramBotDialogueState::ReleasePlan(
                        TelegramBotReleasePlanState::SelectRepositories {
                            date_iso: date_iso.clone(),
                            selected: selected.clone(),
                        },
                    ))
                    .await?;
                edit_repos_menu(&bot, chat_id, message_id, &executors, &selected).await?;
                return Ok(());
            }
            TelegramBotReleasePlanReposAction::Done => {
                if selected.is_empty() {
                    return Ok(());
                }
                let planned_date = match NaiveDate::parse_from_str(&date_iso, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(_) => {
                        edit_menu(
                            &bot,
                            chat_id,
                            message_id,
                            &t!("telegram_bot.dialogues.release_plan.invalid_date").to_string(),
                            None,
                        )
                        .await?;
                        dialogue.exit().await.ok();
                        return Ok(());
                    }
                };

                dialogue
                    .update(TelegramBotDialogueState::ReleasePlan(
                        TelegramBotReleasePlanState::ChooseCallSetup {
                            date_iso: date_iso.clone(),
                            selected: selected.clone(),
                        },
                    ))
                    .await?;

                let default_call_utc = resolve_default_call(planned_date, &config);
                let default_label = format_default_call_label(default_call_utc);
                let kb = build_call_setup_menu(&default_label);
                edit_menu(
                    &bot,
                    chat_id,
                    message_id,
                    &t!("telegram_bot.dialogues.release_plan.choose_call").to_string(),
                    Some(kb),
                )
                .await?;
                return Ok(());
            }
        }
    }

    Ok(())
}

async fn handle_call_setup(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    config: Arc<ApplicationConfig>,
    query: CallbackQuery,
    (date_iso, selected): (String, Vec<i32>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    let action = match TelegramBotReleasePlanCallSetupAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    match action {
        TelegramBotReleasePlanCallSetupAction::Cancel => {
            close_menu(&bot, chat_id, message_id).await;
            dialogue.exit().await.ok();
        }
        TelegramBotReleasePlanCallSetupAction::UseDefault => {
            let planned_date = match NaiveDate::parse_from_str(&date_iso, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    dialogue.exit().await.ok();
                    return Ok(());
                }
            };
            let default_call_utc = resolve_default_call(planned_date, &config);
            let call_iso = default_call_utc.to_rfc3339();
            dialogue
                .update(TelegramBotDialogueState::ReleasePlan(
                    TelegramBotReleasePlanState::EnterMeetingUrl {
                        date_iso: date_iso.clone(),
                        selected: selected.clone(),
                        call_datetime_iso: call_iso,
                    },
                ))
                .await?;

            let kb = build_optional_menu();
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan.enter_meeting_url").to_string(),
                Some(kb),
            )
            .await?;
        }
        TelegramBotReleasePlanCallSetupAction::EnterManually => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlan(
                    TelegramBotReleasePlanState::EnterCallDate {
                        date_iso: date_iso.clone(),
                        selected: selected.clone(),
                    },
                ))
                .await?;

            let kb = build_cancel_only_menu(TelegramBotReleasePlanOptionalAction::Cancel);
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan.enter_call_date").to_string(),
                Some(kb),
            )
            .await?;
        }
    }

    Ok(())
}

async fn handle_call_date_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (date_iso, selected): (String, Vec<i32>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or("").trim();
    let date = match NaiveDate::parse_from_str(text, "%d.%m.%Y") {
        Ok(d) => d,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.release_plan.invalid_date").to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    dialogue
        .update(TelegramBotDialogueState::ReleasePlan(
            TelegramBotReleasePlanState::EnterCallTime {
                date_iso,
                selected,
                call_date_iso: date.format("%Y-%m-%d").to_string(),
            },
        ))
        .await?;

    bot.send_message(
        msg.chat.id,
        t!("telegram_bot.dialogues.release_plan.enter_call_time").to_string(),
    )
    .await?;
    Ok(())
}

async fn handle_call_date_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    query: CallbackQuery,
    _state: (String, Vec<i32>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    if let Ok(TelegramBotReleasePlanOptionalAction::Cancel) =
        TelegramBotReleasePlanOptionalAction::from_callback_data(data)
    {
        close_menu(&bot, chat_id, message_id).await;
        dialogue.exit().await.ok();
    }
    Ok(())
}

async fn handle_call_time_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (date_iso, selected, call_date_iso): (String, Vec<i32>, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or("").trim();
    let time = match NaiveTime::parse_from_str(text, "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.release_plan.invalid_time").to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    let call_date = match NaiveDate::parse_from_str(&call_date_iso, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let local = match Moscow.from_local_datetime(&call_date.and_time(time)).single() {
        Some(dt) => dt,
        None => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.release_plan.invalid_time").to_string(),
            )
            .await?;
            return Ok(());
        }
    };
    let call_utc: DateTime<Utc> = local.with_timezone(&Utc);

    dialogue
        .update(TelegramBotDialogueState::ReleasePlan(
            TelegramBotReleasePlanState::EnterMeetingUrl {
                date_iso,
                selected,
                call_datetime_iso: call_utc.to_rfc3339(),
            },
        ))
        .await?;

    let kb = build_optional_menu();
    bot.send_message(
        msg.chat.id,
        t!("telegram_bot.dialogues.release_plan.enter_meeting_url").to_string(),
    )
    .reply_markup(kb)
    .await?;
    Ok(())
}

async fn handle_call_time_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    query: CallbackQuery,
    _state: (String, Vec<i32>, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    if let Ok(TelegramBotReleasePlanOptionalAction::Cancel) =
        TelegramBotReleasePlanOptionalAction::from_callback_data(data)
    {
        close_menu(&bot, chat_id, message_id).await;
        dialogue.exit().await.ok();
    }
    Ok(())
}

async fn handle_meeting_url_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    (date_iso, selected, call_datetime_iso): (String, Vec<i32>, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = msg.text().unwrap_or("").trim().to_string();
    if url.is_empty() {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan.enter_meeting_url").to_string(),
        )
        .await?;
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReleasePlan(
            TelegramBotReleasePlanState::EnterNote {
                date_iso,
                selected,
                call_datetime_iso,
                meeting_url: Some(url),
            },
        ))
        .await?;

    let kb = build_optional_menu();
    bot.send_message(
        msg.chat.id,
        t!("telegram_bot.dialogues.release_plan.enter_note").to_string(),
    )
    .reply_markup(kb)
    .await?;
    Ok(())
}

async fn handle_meeting_url_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    query: CallbackQuery,
    (date_iso, selected, call_datetime_iso): (String, Vec<i32>, String),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    let action = match TelegramBotReleasePlanOptionalAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    match action {
        TelegramBotReleasePlanOptionalAction::Cancel => {
            close_menu(&bot, chat_id, message_id).await;
            dialogue.exit().await.ok();
        }
        TelegramBotReleasePlanOptionalAction::Skip => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlan(
                    TelegramBotReleasePlanState::EnterNote {
                        date_iso,
                        selected,
                        call_datetime_iso,
                        meeting_url: None,
                    },
                ))
                .await?;

            let kb = build_optional_menu();
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan.enter_note").to_string(),
                Some(kb),
            )
            .await?;
        }
    }

    Ok(())
}

async fn handle_note_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    (date_iso, selected, call_datetime_iso, meeting_url): (
        String,
        Vec<i32>,
        String,
        Option<String>,
    ),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let note_text = msg.text().unwrap_or("").trim().to_string();
    let note_opt = if note_text.is_empty() {
        None
    } else {
        Some(note_text)
    };

    let from = msg.from().cloned();
    let social_user_id =
        from.map(|u| SocialUserId(u.id.0 as i32)).unwrap_or(SocialUserId(0));

    submit_plan(
        &bot,
        msg.chat.id,
        None,
        &dialogue,
        &executors,
        social_user_id,
        date_iso,
        selected,
        call_datetime_iso,
        meeting_url,
        note_opt,
    )
    .await
}

async fn handle_note_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    (date_iso, selected, call_datetime_iso, meeting_url): (
        String,
        Vec<i32>,
        String,
        Option<String>,
    ),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();
    let social_user_id = SocialUserId(query.from.id.0 as i32);

    let action = match TelegramBotReleasePlanOptionalAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    match action {
        TelegramBotReleasePlanOptionalAction::Cancel => {
            close_menu(&bot, chat_id, message_id).await;
            dialogue.exit().await.ok();
            Ok(())
        }
        TelegramBotReleasePlanOptionalAction::Skip => {
            submit_plan(
                &bot,
                chat_id,
                Some(message_id),
                &dialogue,
                &executors,
                social_user_id,
                date_iso,
                selected,
                call_datetime_iso,
                meeting_url,
                None,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn submit_plan(
    bot: &Bot,
    chat_id: ChatId,
    edit_message_id: Option<MessageId>,
    dialogue: &TelegramBotDialogueType,
    executors: &Arc<ApplicationBoostrapExecutors>,
    social_user_id: SocialUserId,
    date_iso: String,
    selected: Vec<i32>,
    call_datetime_iso: String,
    meeting_url: Option<String>,
    note: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let planned_date = match NaiveDate::parse_from_str(&date_iso, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            send_or_edit(
                bot,
                chat_id,
                edit_message_id,
                &t!("telegram_bot.dialogues.release_plan.invalid_date").to_string(),
            )
            .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let call_dt = match DateTime::parse_from_rfc3339(&call_datetime_iso) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => {
            send_or_edit(
                bot,
                chat_id,
                edit_message_id,
                &t!("telegram_bot.dialogues.release_plan.error").to_string(),
            )
            .await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    };

    let cmd = CreateReleasePlanExecutorCommand {
        created_by_social_user_id: social_user_id,
        planned_date,
        call_datetime: Some(call_dt),
        meeting_url,
        note,
        announce_chat_id: None,
        repository_ids: selected.iter().map(|id| RepositoryId(*id)).collect(),
    };

    let final_text = match executors.commands.create_release_plan.execute(&cmd).await {
        Ok(r) => t!(
            "telegram_bot.dialogues.release_plan.created",
            id = r.plan.id.0
        )
        .to_string(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create release plan");
            t!("telegram_bot.dialogues.release_plan.error").to_string()
        }
    };

    send_or_edit(bot, chat_id, edit_message_id, &final_text).await?;
    dialogue.exit().await.ok();
    Ok(())
}

async fn send_or_edit(
    bot: &Bot,
    chat_id: ChatId,
    message_id: Option<MessageId>,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match message_id {
        Some(mid) => edit_menu(bot, chat_id, mid, text, None).await,
        None => {
            bot.send_message(chat_id, text).await?;
            Ok(())
        }
    }
}
