use crate::application::release_plan::commands::create_release_plan::command::CreateReleasePlanExecutorCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::dialogues::helpers::{close_menu, edit_menu};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::release_plan::{
    REPO_TOGGLE_PREFIX, TelegramBotReleasePlanReposAction, repo_toggle_callback,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use chrono::NaiveDate;
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
}

pub struct TelegramBotReleasePlanDispatcher {}

impl TelegramBotReleasePlanDispatcher {
    pub fn new()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        let messages = Update::filter_message()
            .branch(case![TelegramBotReleasePlanState::EnterDate].endpoint(handle_date_input));

        let queries = Update::filter_callback_query().branch(
            case![TelegramBotReleasePlanState::SelectRepositories { date_iso, selected }]
                .endpoint(handle_repos),
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
    let social_user_id = SocialUserId(query.from.id.0 as i32);

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

                let cmd = CreateReleasePlanExecutorCommand {
                    created_by_social_user_id: social_user_id,
                    planned_date,
                    call_datetime: None,
                    meeting_url: None,
                    note: None,
                    // Executor сам резолвит чат из первого репо (notifications_chat_id или social_chat_id)
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
                edit_menu(&bot, chat_id, message_id, &final_text, None).await?;
                dialogue.exit().await.ok();
                return Ok(());
            }
        }
    }

    Ok(())
}
