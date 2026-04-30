use crate::application::release_plan::commands::cancel_release_plan::command::CancelReleasePlanExecutorCommand;
use crate::application::release_plan::commands::complete_release_plan::command::CompleteReleasePlanExecutorCommand;
use crate::application::release_plan::commands::update_release_plan::command::{
    ReleasePlanPatch, UpdateReleasePlanExecutorCommand,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::helpers::{close_menu, edit_menu};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::release_plan_settings::{
    RPS_CANCEL_BTN_PREFIX, RPS_COMPLETE_BTN_PREFIX, RPS_REPO_TOGGLE_PREFIX, RPS_SELECT_PREFIX,
    TelegramBotReleasePlanSettingsConfirmAction, TelegramBotReleasePlanSettingsMenuAction,
    TelegramBotReleasePlanSettingsReposAction, rps_repo_toggle_callback,
};
use crate::domain::release_plan::value_objects::release_plan_id::ReleasePlanId;
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
pub enum TelegramBotReleasePlanSettingsState {
    #[default]
    AwaitingSelection,
    Menu {
        plan_id: i32,
    },
    EnterPlannedDate {
        plan_id: i32,
    },
    EnterCallDate {
        plan_id: i32,
    },
    EnterCallTime {
        plan_id: i32,
        call_date_iso: String,
    },
    EnterMeetingUrl {
        plan_id: i32,
    },
    EnterNote {
        plan_id: i32,
    },
    SelectRepositories {
        plan_id: i32,
        selected: Vec<i32>,
    },
    EnterCancelReason {
        plan_id: i32,
    },
    ConfirmCancel {
        plan_id: i32,
        reason: String,
    },
    ConfirmComplete {
        plan_id: i32,
    },
}

pub struct TelegramBotReleasePlanSettingsDispatcher {}

impl TelegramBotReleasePlanSettingsDispatcher {
    pub fn new()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        let messages = Update::filter_message()
            .branch(
                case![TelegramBotReleasePlanSettingsState::EnterPlannedDate { plan_id }]
                    .endpoint(handle_planned_date_input),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::EnterCallDate { plan_id }]
                    .endpoint(handle_call_date_input),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::EnterCallTime {
                    plan_id,
                    call_date_iso
                }]
                .endpoint(handle_call_time_input),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::EnterMeetingUrl { plan_id }]
                    .endpoint(handle_meeting_url_input),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::EnterNote { plan_id }]
                    .endpoint(handle_note_input),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::EnterCancelReason { plan_id }]
                    .endpoint(handle_cancel_reason_input),
            );

        let queries = Update::filter_callback_query()
            .branch(
                case![TelegramBotReleasePlanSettingsState::AwaitingSelection]
                    .endpoint(handle_awaiting_selection),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::Menu { plan_id }]
                    .endpoint(handle_menu_callback),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::SelectRepositories {
                    plan_id,
                    selected
                }]
                .endpoint(handle_select_repositories_callback),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::ConfirmCancel { plan_id, reason }]
                    .endpoint(handle_confirm_cancel_callback),
            )
            .branch(
                case![TelegramBotReleasePlanSettingsState::ConfirmComplete { plan_id }]
                    .endpoint(handle_confirm_complete_callback),
            );

        teloxide::dptree::entry().branch(messages).branch(queries)
    }
}

fn build_menu_keyboard() -> InlineKeyboardMarkup {
    use TelegramBotReleasePlanSettingsMenuAction::*;
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            EditPlannedDate.label(),
            EditPlannedDate.to_callback_data(),
        )],
        vec![
            InlineKeyboardButton::callback(EditCallDate.label(), EditCallDate.to_callback_data()),
            InlineKeyboardButton::callback(RemoveCall.label(), RemoveCall.to_callback_data()),
        ],
        vec![
            InlineKeyboardButton::callback(EditMeeting.label(), EditMeeting.to_callback_data()),
            InlineKeyboardButton::callback(RemoveMeeting.label(), RemoveMeeting.to_callback_data()),
        ],
        vec![
            InlineKeyboardButton::callback(EditNote.label(), EditNote.to_callback_data()),
            InlineKeyboardButton::callback(RemoveNote.label(), RemoveNote.to_callback_data()),
        ],
        vec![InlineKeyboardButton::callback(
            EditRepos.label(),
            EditRepos.to_callback_data(),
        )],
        vec![InlineKeyboardButton::callback(
            Close.label(),
            Close.to_callback_data(),
        )],
    ])
}

fn build_confirm_keyboard() -> InlineKeyboardMarkup {
    use TelegramBotReleasePlanSettingsConfirmAction::*;
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(Yes.label(), Yes.to_callback_data()),
        InlineKeyboardButton::callback(No.label(), No.to_callback_data()),
    ]])
}

async fn render_menu_text(
    executors: &Arc<ApplicationBoostrapExecutors>,
    plan_id: i32,
) -> String {
    use crate::utils::builder::message::MessageBuilder;
    let plan = match executors
        .queries
        .get_upcoming_release_plans
        .execute(&crate::application::release_plan::queries::get_upcoming_release_plans::query::GetUpcomingReleasePlansQuery {
            from_date: chrono::NaiveDate::MIN,
        })
        .await
    {
        Ok(r) => r.plans.into_iter().find(|p| p.id.0 == plan_id),
        Err(_) => None,
    };

    let Some(plan) = plan else {
        return t!("telegram_bot.dialogues.release_plan_settings.not_found").to_string();
    };

    let repos = executors
        .queries
        .get_all_repositories
        .execute(&crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery {})
        .await
        .map(|r| r.repositories)
        .unwrap_or_default();
    let repo_label_by_id: std::collections::HashMap<RepositoryId, String> = repos
        .into_iter()
        .map(|r| (r.id, format!("{}/{}", r.owner, r.name)))
        .collect();

    let repos_text = if plan.repository_ids.is_empty() {
        "—".to_string()
    } else {
        plan.repository_ids
            .iter()
            .map(|id| {
                repo_label_by_id
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| format!("#{}", id.0))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    let call_text = match plan.call_datetime {
        Some(c) => c
            .with_timezone(&Moscow)
            .format("%d.%m %H:%M МСК")
            .to_string(),
        None => "—".to_string(),
    };

    let meeting_text = plan.meeting_url.clone().unwrap_or_else(|| "—".to_string());
    let note_text = plan.note.clone().unwrap_or_else(|| "—".to_string());

    MessageBuilder::new()
        .bold(
            &t!(
                "telegram_bot.dialogues.release_plan_settings.menu_title",
                date = plan.planned_date.format("%d.%m.%Y").to_string()
            )
            .to_string(),
        )
        .empty_line()
        .with_html_escape(true)
        .section("📦 Репо", &repos_text)
        .section("🕐 Созвон", &call_text)
        .section("🔗 Ссылка", &meeting_text)
        .section("📝 Заметка", &note_text)
        .build()
}

async fn show_menu(
    bot: &Bot,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    message_id: MessageId,
    plan_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = render_menu_text(executors, plan_id).await;
    let kb = build_menu_keyboard();
    edit_menu(bot, chat_id, message_id, &text, Some(kb)).await
}

async fn handle_awaiting_selection(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    if let Some(rest) = data.strip_prefix(RPS_SELECT_PREFIX) {
        if let Ok(plan_id) = rest.parse::<i32>() {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::Menu { plan_id },
                ))
                .await?;
            show_menu(&bot, &executors, chat_id, message_id, plan_id).await?;
            return Ok(());
        }
    }

    if let Some(rest) = data.strip_prefix(RPS_CANCEL_BTN_PREFIX) {
        if let Ok(plan_id) = rest.parse::<i32>() {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::EnterCancelReason { plan_id },
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan_settings.enter_cancel_reason")
                    .to_string(),
                None,
            )
            .await?;
            return Ok(());
        }
    }

    if let Some(rest) = data.strip_prefix(RPS_COMPLETE_BTN_PREFIX) {
        if let Ok(plan_id) = rest.parse::<i32>() {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::ConfirmComplete { plan_id },
                ))
                .await?;
            let kb = build_confirm_keyboard();
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan_settings.confirm_complete")
                    .to_string(),
                Some(kb),
            )
            .await?;
            return Ok(());
        }
    }

    Ok(())
}

async fn handle_menu_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    plan_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    let action = match TelegramBotReleasePlanSettingsMenuAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    use TelegramBotReleasePlanSettingsMenuAction::*;
    match action {
        Close => {
            close_menu(&bot, chat_id, message_id).await;
            dialogue.exit().await.ok();
        }
        EditPlannedDate => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::EnterPlannedDate { plan_id },
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan_settings.enter_planned_date")
                    .to_string(),
                None,
            )
            .await?;
        }
        EditCallDate => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::EnterCallDate { plan_id },
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan.enter_call_date").to_string(),
                None,
            )
            .await?;
        }
        EditMeeting => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::EnterMeetingUrl { plan_id },
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan_settings.enter_meeting_url")
                    .to_string(),
                None,
            )
            .await?;
        }
        EditNote => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::EnterNote { plan_id },
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.release_plan_settings.enter_note").to_string(),
                None,
            )
            .await?;
        }
        RemoveCall => {
            apply_patch_and_refresh(
                &bot,
                &dialogue,
                &executors,
                chat_id,
                message_id,
                plan_id,
                ReleasePlanPatch::ClearCallDateTime,
            )
            .await?;
        }
        RemoveMeeting => {
            apply_patch_and_refresh(
                &bot,
                &dialogue,
                &executors,
                chat_id,
                message_id,
                plan_id,
                ReleasePlanPatch::ClearMeetingUrl,
            )
            .await?;
        }
        RemoveNote => {
            apply_patch_and_refresh(
                &bot,
                &dialogue,
                &executors,
                chat_id,
                message_id,
                plan_id,
                ReleasePlanPatch::ClearNote,
            )
            .await?;
        }
        EditRepos => {
            let plan = match executors
                .queries
                .get_upcoming_release_plans
                .execute(&crate::application::release_plan::queries::get_upcoming_release_plans::query::GetUpcomingReleasePlansQuery {
                    from_date: chrono::NaiveDate::MIN,
                })
                .await
            {
                Ok(r) => r.plans.into_iter().find(|p| p.id.0 == plan_id),
                Err(_) => None,
            };

            let selected: Vec<i32> = plan
                .map(|p| p.repository_ids.iter().map(|r| r.0).collect())
                .unwrap_or_default();

            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::SelectRepositories {
                        plan_id,
                        selected: selected.clone(),
                    },
                ))
                .await?;
            edit_repos_select(&bot, chat_id, message_id, &executors, &selected).await?;
        }
    }

    Ok(())
}

async fn apply_patch_and_refresh(
    bot: &Bot,
    dialogue: &TelegramBotDialogueType,
    executors: &Arc<ApplicationBoostrapExecutors>,
    chat_id: ChatId,
    message_id: MessageId,
    plan_id: i32,
    patch: ReleasePlanPatch,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Err(e) = executors
        .commands
        .update_release_plan
        .execute(&UpdateReleasePlanExecutorCommand {
            plan_id: ReleasePlanId(plan_id),
            patch,
        })
        .await
    {
        tracing::error!(error = %e, plan_id, "Failed to update release plan");
        edit_menu(
            bot,
            chat_id,
            message_id,
            &t!("telegram_bot.dialogues.release_plan_settings.error").to_string(),
            None,
        )
        .await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReleasePlanSettings(
            TelegramBotReleasePlanSettingsState::Menu { plan_id },
        ))
        .await?;
    show_menu(bot, executors, chat_id, message_id, plan_id).await?;
    Ok(())
}

async fn handle_planned_date_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    plan_id: i32,
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

    if let Err(e) = executors
        .commands
        .update_release_plan
        .execute(&UpdateReleasePlanExecutorCommand {
            plan_id: ReleasePlanId(plan_id),
            patch: ReleasePlanPatch::SetPlannedDate { date },
        })
        .await
    {
        tracing::error!(error = %e, "Failed to update planned date");
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan_settings.error").to_string(),
        )
        .await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReleasePlanSettings(
            TelegramBotReleasePlanSettingsState::Menu { plan_id },
        ))
        .await?;
    let text = render_menu_text(&executors, plan_id).await;
    let kb = build_menu_keyboard();
    bot.send_message(msg.chat.id, text)
        .reply_markup(kb)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn handle_call_date_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    plan_id: i32,
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
        .update(TelegramBotDialogueState::ReleasePlanSettings(
            TelegramBotReleasePlanSettingsState::EnterCallTime {
                plan_id,
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

async fn handle_call_time_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    (plan_id, call_date_iso): (i32, String),
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

    if let Err(e) = executors
        .commands
        .update_release_plan
        .execute(&UpdateReleasePlanExecutorCommand {
            plan_id: ReleasePlanId(plan_id),
            patch: ReleasePlanPatch::SetCallDateTime {
                datetime: call_utc,
            },
        })
        .await
    {
        tracing::error!(error = %e, "Failed to update call datetime");
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan_settings.error").to_string(),
        )
        .await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReleasePlanSettings(
            TelegramBotReleasePlanSettingsState::Menu { plan_id },
        ))
        .await?;
    let text = render_menu_text(&executors, plan_id).await;
    let kb = build_menu_keyboard();
    bot.send_message(msg.chat.id, text)
        .reply_markup(kb)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn handle_meeting_url_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    plan_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = msg.text().unwrap_or("").trim().to_string();
    if url.is_empty() {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan_settings.enter_meeting_url").to_string(),
        )
        .await?;
        return Ok(());
    }

    if let Err(e) = executors
        .commands
        .update_release_plan
        .execute(&UpdateReleasePlanExecutorCommand {
            plan_id: ReleasePlanId(plan_id),
            patch: ReleasePlanPatch::SetMeetingUrl { url },
        })
        .await
    {
        tracing::error!(error = %e, "Failed to update meeting url");
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan_settings.error").to_string(),
        )
        .await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReleasePlanSettings(
            TelegramBotReleasePlanSettingsState::Menu { plan_id },
        ))
        .await?;
    let text = render_menu_text(&executors, plan_id).await;
    let kb = build_menu_keyboard();
    bot.send_message(msg.chat.id, text)
        .reply_markup(kb)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn handle_note_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    plan_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let note = msg.text().unwrap_or("").trim().to_string();
    if note.is_empty() {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan_settings.enter_note").to_string(),
        )
        .await?;
        return Ok(());
    }

    if let Err(e) = executors
        .commands
        .update_release_plan
        .execute(&UpdateReleasePlanExecutorCommand {
            plan_id: ReleasePlanId(plan_id),
            patch: ReleasePlanPatch::SetNote { text: note },
        })
        .await
    {
        tracing::error!(error = %e, "Failed to update note");
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan_settings.error").to_string(),
        )
        .await?;
        dialogue.exit().await.ok();
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReleasePlanSettings(
            TelegramBotReleasePlanSettingsState::Menu { plan_id },
        ))
        .await?;
    let text = render_menu_text(&executors, plan_id).await;
    let kb = build_menu_keyboard();
    bot.send_message(msg.chat.id, text)
        .reply_markup(kb)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn build_repos_select_keyboard(
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
            rps_repo_toggle_callback(repo.id.0),
        )]);
    }
    rows.push(vec![
        InlineKeyboardButton::callback(
            TelegramBotReleasePlanSettingsReposAction::Save.label(),
            TelegramBotReleasePlanSettingsReposAction::Save.to_callback_data(),
        ),
        InlineKeyboardButton::callback(
            TelegramBotReleasePlanSettingsReposAction::Cancel.label(),
            TelegramBotReleasePlanSettingsReposAction::Cancel.to_callback_data(),
        ),
    ]);
    InlineKeyboardMarkup::new(rows)
}

async fn edit_repos_select(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    selected: &[i32],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kb = build_repos_select_keyboard(executors, selected).await;
    edit_menu(
        bot,
        chat_id,
        message_id,
        &t!("telegram_bot.dialogues.release_plan_settings.select_repos").to_string(),
        Some(kb),
    )
    .await
}

async fn handle_select_repositories_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    (plan_id, mut selected): (i32, Vec<i32>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    if let Some(rest) = data.strip_prefix(RPS_REPO_TOGGLE_PREFIX) {
        if let Ok(repo_id) = rest.parse::<i32>() {
            if let Some(pos) = selected.iter().position(|x| *x == repo_id) {
                selected.remove(pos);
            } else {
                selected.push(repo_id);
            }
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::SelectRepositories {
                        plan_id,
                        selected: selected.clone(),
                    },
                ))
                .await?;
            edit_repos_select(&bot, chat_id, message_id, &executors, &selected).await?;
            return Ok(());
        }
    }

    if let Ok(action) = TelegramBotReleasePlanSettingsReposAction::from_callback_data(data) {
        match action {
            TelegramBotReleasePlanSettingsReposAction::Cancel => {
                dialogue
                    .update(TelegramBotDialogueState::ReleasePlanSettings(
                        TelegramBotReleasePlanSettingsState::Menu { plan_id },
                    ))
                    .await?;
                show_menu(&bot, &executors, chat_id, message_id, plan_id).await?;
            }
            TelegramBotReleasePlanSettingsReposAction::Save => {
                if selected.is_empty() {
                    return Ok(());
                }
                let ids: Vec<RepositoryId> =
                    selected.iter().map(|id| RepositoryId(*id)).collect();
                if let Err(e) = executors
                    .commands
                    .update_release_plan
                    .execute(&UpdateReleasePlanExecutorCommand {
                        plan_id: ReleasePlanId(plan_id),
                        patch: ReleasePlanPatch::SetRepositories { ids },
                    })
                    .await
                {
                    tracing::error!(error = %e, "Failed to update repositories");
                    edit_menu(
                        &bot,
                        chat_id,
                        message_id,
                        &t!("telegram_bot.dialogues.release_plan_settings.error").to_string(),
                        None,
                    )
                    .await?;
                    dialogue.exit().await.ok();
                    return Ok(());
                }
                dialogue
                    .update(TelegramBotDialogueState::ReleasePlanSettings(
                        TelegramBotReleasePlanSettingsState::Menu { plan_id },
                    ))
                    .await?;
                show_menu(&bot, &executors, chat_id, message_id, plan_id).await?;
            }
        }
    }

    Ok(())
}

async fn handle_cancel_reason_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    msg: Message,
    plan_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let reason = msg.text().unwrap_or("").trim().to_string();
    if reason.is_empty() {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.release_plan_settings.enter_cancel_reason").to_string(),
        )
        .await?;
        return Ok(());
    }

    dialogue
        .update(TelegramBotDialogueState::ReleasePlanSettings(
            TelegramBotReleasePlanSettingsState::ConfirmCancel {
                plan_id,
                reason: reason.clone(),
            },
        ))
        .await?;

    let kb = build_confirm_keyboard();
    bot.send_message(
        msg.chat.id,
        t!(
            "telegram_bot.dialogues.release_plan_settings.confirm_cancel",
            reason = reason
        )
        .to_string(),
    )
    .reply_markup(kb)
    .await?;
    Ok(())
}

async fn handle_confirm_cancel_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    (plan_id, reason): (i32, String),
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

    let action = match TelegramBotReleasePlanSettingsConfirmAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    match action {
        TelegramBotReleasePlanSettingsConfirmAction::No => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::Menu { plan_id },
                ))
                .await?;
            show_menu(&bot, &executors, chat_id, message_id, plan_id).await?;
        }
        TelegramBotReleasePlanSettingsConfirmAction::Yes => {
            let final_text = match executors
                .commands
                .cancel_release_plan
                .execute(&CancelReleasePlanExecutorCommand {
                    plan_id: ReleasePlanId(plan_id),
                    cancelled_by_social_user_id: social_user_id,
                    reason,
                })
                .await
            {
                Ok(_) => t!("telegram_bot.dialogues.release_plan_settings.cancelled").to_string(),
                Err(e) => {
                    tracing::error!(error = %e, "Failed to cancel release plan");
                    t!("telegram_bot.dialogues.release_plan_settings.error").to_string()
                }
            };
            edit_menu(&bot, chat_id, message_id, &final_text, None).await?;
            dialogue.exit().await.ok();
        }
    }
    Ok(())
}

async fn handle_confirm_complete_callback(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    plan_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;
    let data = query.data.as_deref().unwrap_or("");
    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    let chat_id = msg.chat().id;
    let message_id = msg.id();

    let action = match TelegramBotReleasePlanSettingsConfirmAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    match action {
        TelegramBotReleasePlanSettingsConfirmAction::No => {
            dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::Menu { plan_id },
                ))
                .await?;
            show_menu(&bot, &executors, chat_id, message_id, plan_id).await?;
        }
        TelegramBotReleasePlanSettingsConfirmAction::Yes => {
            let final_text = match executors
                .commands
                .complete_release_plan
                .execute(&CompleteReleasePlanExecutorCommand {
                    plan_id: ReleasePlanId(plan_id),
                })
                .await
            {
                Ok(_) => t!("telegram_bot.dialogues.release_plan_settings.completed").to_string(),
                Err(e) => {
                    tracing::error!(error = %e, "Failed to complete release plan");
                    t!("telegram_bot.dialogues.release_plan_settings.error").to_string()
                }
            };
            edit_menu(&bot, chat_id, message_id, &final_text, None).await?;
            dialogue.exit().await.ok();
        }
    }
    Ok(())
}
