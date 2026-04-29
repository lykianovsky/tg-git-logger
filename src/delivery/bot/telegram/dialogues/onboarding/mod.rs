use crate::application::user::commands::bind_repository::command::BindRepositoryCommand;
use crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery;
use crate::application::user_preferences::commands::update_user_preferences::command::{
    UpdateUserPreferencesExecutorCommand, UserPreferencesPatch,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::helpers::{close_menu, edit_menu};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::onboarding::{
    REPO_TOGGLE_PREFIX, TelegramBotOnboardingDndAction, TelegramBotOnboardingReposAction,
    repo_toggle_callback,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use chrono::NaiveTime;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotOnboardingState {
    #[default]
    SelectRepositories,
    ChooseDndWindow,
    EnterDnd,
}

pub struct TelegramBotOnboardingDispatcher {}

impl TelegramBotOnboardingDispatcher {
    pub fn new()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query()
            .branch(case![TelegramBotOnboardingState::SelectRepositories].endpoint(handle_repos))
            .branch(case![TelegramBotOnboardingState::ChooseDndWindow].endpoint(handle_dnd_choice));

        let messages = Update::filter_message()
            .branch(case![TelegramBotOnboardingState::EnterDnd].endpoint(handle_dnd_input));

        teloxide::dptree::entry().branch(queries).branch(messages)
    }
}

async fn build_repos_menu(
    executors: &Arc<ApplicationBoostrapExecutors>,
    social_user_id: SocialUserId,
) -> Option<InlineKeyboardMarkup> {
    let bound = executors
        .queries
        .get_user_bound_repositories
        .execute(&GetUserBoundRepositoriesQuery { social_user_id })
        .await
        .ok()?;

    let all_repos = executors
        .queries
        .get_all_repositories
        .execute(&crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery {})
        .await
        .map(|r| r.repositories)
        .unwrap_or_default();

    let bound_ids: std::collections::HashSet<i32> =
        bound.repositories.iter().map(|r| r.id.0).collect();

    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for repo in &all_repos {
        let mark = if bound_ids.contains(&repo.id.0) {
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
        TelegramBotOnboardingReposAction::SelectAll
            .label()
            .to_string(),
        TelegramBotOnboardingReposAction::SelectAll
            .to_callback_data()
            .to_string(),
    )]);
    rows.push(vec![
        InlineKeyboardButton::callback(
            TelegramBotOnboardingReposAction::Done.label().to_string(),
            TelegramBotOnboardingReposAction::Done
                .to_callback_data()
                .to_string(),
        ),
        InlineKeyboardButton::callback(
            TelegramBotOnboardingReposAction::Skip.label().to_string(),
            TelegramBotOnboardingReposAction::Skip
                .to_callback_data()
                .to_string(),
        ),
    ]);

    Some(InlineKeyboardMarkup::new(rows))
}

pub async fn send_repos_menu(
    bot: &Bot,
    chat_id: ChatId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kb = match build_repos_menu(executors, social_user_id).await {
        Some(k) => k,
        None => {
            bot.send_message(
                chat_id,
                t!("telegram_bot.dialogues.onboarding.repos_load_error").to_string(),
            )
            .await?;
            return Ok(());
        }
    };
    bot.send_message(
        chat_id,
        t!("telegram_bot.dialogues.onboarding.select_repos").to_string(),
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
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kb = match build_repos_menu(executors, social_user_id).await {
        Some(k) => k,
        None => return Ok(()),
    };
    let text = t!("telegram_bot.dialogues.onboarding.select_repos").to_string();
    edit_menu(bot, chat_id, message_id, &text, Some(kb)).await
}

async fn handle_repos(
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
    let social_user_id = SocialUserId(query.from.id.0 as i32);

    let chat_id = msg.chat().id;
    let message_id = msg.id();

    if let Some(repo_id_str) = data.strip_prefix(REPO_TOGGLE_PREFIX) {
        if let Ok(repo_id) = repo_id_str.parse::<i32>() {
            let bind_cmd = BindRepositoryCommand {
                social_user_id,
                repository_id: RepositoryId(repo_id),
            };
            match executors.commands.bind_repository.execute(&bind_cmd).await {
                Ok(_) => {
                    tracing::debug!(repo_id, "Onboarding: bound repository");
                }
                Err(crate::application::user::commands::bind_repository::error::BindRepositoryExecutorError::AlreadyBound) => {
                    let unbind_cmd =
                        crate::application::user::commands::unbind_repository::command::UnbindRepositoryCommand {
                            social_user_id,
                            repository_id: RepositoryId(repo_id),
                        };
                    if let Err(e) =
                        executors.commands.unbind_repository.execute(&unbind_cmd).await
                    {
                        tracing::warn!(error = %e, "Onboarding: failed to unbind toggled repo");
                    } else {
                        tracing::debug!(repo_id, "Onboarding: unbound repository");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Onboarding: bind failed");
                }
            }
            edit_repos_menu(&bot, chat_id, message_id, &executors, social_user_id).await?;
        }
        return Ok(());
    }

    if let Ok(action) = TelegramBotOnboardingReposAction::from_callback_data(data) {
        match action {
            TelegramBotOnboardingReposAction::SelectAll => {
                let all = executors
                    .queries
                    .get_all_repositories
                    .execute(&crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery {})
                    .await
                    .ok();
                if let Some(r) = all {
                    for repo in r.repositories {
                        let _ = executors
                            .commands
                            .bind_repository
                            .execute(&BindRepositoryCommand {
                                social_user_id,
                                repository_id: repo.id,
                            })
                            .await;
                    }
                }
                edit_repos_menu(&bot, chat_id, message_id, &executors, social_user_id).await?;
            }
            TelegramBotOnboardingReposAction::Done | TelegramBotOnboardingReposAction::Skip => {
                go_to_dnd_step(&bot, chat_id, message_id, &dialogue).await?;
            }
        }
    }

    Ok(())
}

async fn go_to_dnd_step(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    dialogue: &TelegramBotDialogueType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rows = vec![
        vec![InlineKeyboardButton::callback(
            TelegramBotOnboardingDndAction::KeepDefault
                .label()
                .to_string(),
            TelegramBotOnboardingDndAction::KeepDefault
                .to_callback_data()
                .to_string(),
        )],
        vec![
            InlineKeyboardButton::callback(
                TelegramBotOnboardingDndAction::Custom.label().to_string(),
                TelegramBotOnboardingDndAction::Custom
                    .to_callback_data()
                    .to_string(),
            ),
            InlineKeyboardButton::callback(
                TelegramBotOnboardingDndAction::Skip.label().to_string(),
                TelegramBotOnboardingDndAction::Skip
                    .to_callback_data()
                    .to_string(),
            ),
        ],
    ];

    dialogue
        .update(TelegramBotDialogueState::Onboarding(
            TelegramBotOnboardingState::ChooseDndWindow,
        ))
        .await?;

    edit_menu(
        bot,
        chat_id,
        message_id,
        &t!("telegram_bot.dialogues.onboarding.choose_dnd").to_string(),
        Some(InlineKeyboardMarkup::new(rows)),
    )
    .await
}

async fn handle_dnd_choice(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
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
    let action = match TelegramBotOnboardingDndAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    match action {
        TelegramBotOnboardingDndAction::KeepDefault | TelegramBotOnboardingDndAction::Skip => {
            finish_onboarding_edit(&bot, chat_id, message_id, &dialogue).await?;
        }
        TelegramBotOnboardingDndAction::Custom => {
            dialogue
                .update(TelegramBotDialogueState::Onboarding(
                    TelegramBotOnboardingState::EnterDnd,
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.onboarding.enter_dnd").to_string(),
                None,
            )
            .await?;
        }
    }

    Ok(())
}

async fn finish_onboarding_edit(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    dialogue: &TelegramBotDialogueType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    edit_menu(
        bot,
        chat_id,
        message_id,
        &t!("telegram_bot.dialogues.onboarding.finished").to_string(),
        None,
    )
    .await?;
    dialogue.exit().await.ok();
    Ok(())
}

async fn handle_dnd_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or("").trim();
    let parts: Vec<&str> = text.split('-').map(|s| s.trim()).collect();

    if parts.len() != 2 {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.onboarding.invalid_dnd").to_string(),
        )
        .await?;
        return Ok(());
    }

    let start = match NaiveTime::parse_from_str(parts[0], "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.onboarding.invalid_dnd").to_string(),
            )
            .await?;
            return Ok(());
        }
    };
    let end = match NaiveTime::parse_from_str(parts[1], "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.onboarding.invalid_dnd").to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    let social_user_id = SocialUserId(msg.from.as_ref().map(|u| u.id.0 as i32).unwrap_or(0));

    if let Err(e) = executors
        .commands
        .update_user_preferences
        .execute(&UpdateUserPreferencesExecutorCommand {
            social_user_id,
            patch: UserPreferencesPatch::SetDndWindow { start, end },
        })
        .await
    {
        tracing::error!(error = %e, "Failed to set DND in onboarding");
    }

    finish_onboarding(&bot, msg.chat.id, &dialogue).await?;
    Ok(())
}

async fn finish_onboarding(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: &TelegramBotDialogueType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.send_message(
        chat_id,
        t!("telegram_bot.dialogues.onboarding.finished").to_string(),
    )
    .await?;
    dialogue.exit().await.ok();
    Ok(())
}
