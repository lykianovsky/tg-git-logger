use crate::application::digest::commands::create_digest_subscription::command::CreateDigestSubscriptionCommand;
use crate::application::digest::commands::delete_digest_subscription::command::DeleteDigestSubscriptionCommand;
use crate::application::digest::commands::toggle_digest_subscription::command::ToggleDigestSubscriptionCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::confirm::TelegramBotConfirmAction;
use crate::delivery::bot::telegram::keyboards::actions::digest_list::TelegramBotDigestListAction;
use crate::delivery::bot::telegram::keyboards::actions::digest_repository::TelegramBotDigestRepositoryAction;
use crate::delivery::bot::telegram::keyboards::actions::digest_type::TelegramBotDigestTypeAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::digest::value_objects::digest_subscription_id::DigestSubscriptionId;
use crate::domain::digest::value_objects::digest_type::DigestType;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{dptree, Bot};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotDigestState {
    #[default]
    List,

    ChooseType,

    ChooseRepository {
        digest_type: DigestType,
    },

    ChooseTime {
        digest_type: DigestType,
        repository_id: Option<i32>,
    },

    ConfirmDelete {
        subscription_id: i32,
    },
}

pub struct TelegramBotDigestDispatcher {}

impl TelegramBotDigestDispatcher {
    pub fn new()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query()
            .branch(case![TelegramBotDigestState::List].endpoint(handle_list_action))
            .branch(case![TelegramBotDigestState::ChooseType].endpoint(handle_choose_type))
            .branch(
                case![TelegramBotDigestState::ChooseRepository { digest_type }]
                    .endpoint(handle_choose_repository),
            )
            .branch(
                case![TelegramBotDigestState::ConfirmDelete { subscription_id }]
                    .endpoint(handle_confirm_delete),
            );

        let messages = Update::filter_message().branch(
            case![TelegramBotDigestState::ChooseTime {
                digest_type,
                repository_id
            }]
            .endpoint(handle_choose_time),
        );

        dptree::entry().branch(queries).branch(messages)
    }
}

async fn handle_list_action(
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

    if TelegramBotDigestListAction::from_callback_data(data)
        .map(|a| matches!(a, TelegramBotDigestListAction::Create))
        .unwrap_or(false)
    {
        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotDigestTypeAction>(vec![TelegramBotDigestTypeAction::Daily])
            .row::<TelegramBotDigestTypeAction>(vec![TelegramBotDigestTypeAction::Weekly])
            .row::<TelegramBotDigestTypeAction>(vec![TelegramBotDigestTypeAction::Cancel])
            .build();

        dialogue
            .update(TelegramBotDialogueState::Digest(
                TelegramBotDigestState::ChooseType,
            ))
            .await?;

        bot.send_message(
            msg.chat().id,
            t!("telegram_bot.dialogues.digest.choose_type").to_string(),
        )
        .reply_markup(keyboard)
        .await?;

        return Ok(());
    }

    if TelegramBotDigestListAction::from_callback_data(data)
        .map(|a| matches!(a, TelegramBotDigestListAction::Cancel))
        .unwrap_or(false)
    {
        bot.send_message(
            msg.chat().id,
            t!("telegram_bot.common.cancelled").to_string(),
        )
        .await?;

        dialogue.exit().await.ok();
        return Ok(());
    }

    if let Some(id_str) = data.strip_prefix(TelegramBotDigestListAction::TOGGLE_PREFIX) {
        if let Ok(id) = id_str.parse::<i32>() {
            let cmd = ToggleDigestSubscriptionCommand {
                id: DigestSubscriptionId(id),
            };

            let reply = match executors
                .commands
                .toggle_digest_subscription
                .execute(&cmd)
                .await
            {
                Ok(result) => {
                    if result.is_active {
                        t!("telegram_bot.dialogues.digest.enabled").to_string()
                    } else {
                        t!("telegram_bot.dialogues.digest.disabled").to_string()
                    }
                }

                Err(e) => {
                    tracing::error!(error = %e, "Failed to toggle digest subscription");
                    t!("telegram_bot.dialogues.digest.error").to_string()
                }
            };

            bot.send_message(msg.chat().id, reply).await?;
            dialogue.exit().await.ok();
            return Ok(());
        }
    }

    if let Some(id_str) = data.strip_prefix(TelegramBotDigestListAction::DELETE_PREFIX) {
        if let Ok(id) = id_str.parse::<i32>() {
            let keyboard = KeyboardBuilder::new()
                .row::<TelegramBotConfirmAction>(vec![
                    TelegramBotConfirmAction::Yes,
                    TelegramBotConfirmAction::No,
                ])
                .build();

            dialogue
                .update(TelegramBotDialogueState::Digest(
                    TelegramBotDigestState::ConfirmDelete {
                        subscription_id: id,
                    },
                ))
                .await?;

            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.dialogues.digest.confirm_delete").to_string(),
            )
            .reply_markup(keyboard)
            .await?;

            return Ok(());
        }
    }

    Ok(())
}

async fn handle_choose_type(
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

    let action = match TelegramBotDigestTypeAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    match action {
        TelegramBotDigestTypeAction::Cancel => {
            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.common.cancelled").to_string(),
            )
            .await?;

            dialogue.exit().await.ok();
            return Ok(());
        }
        TelegramBotDigestTypeAction::Daily | TelegramBotDigestTypeAction::Weekly => {}
    }

    let social_user_id = SocialUserId(query.from.id.0 as i32);

    let bound_repos = executors
        .queries
        .get_user_bound_repositories
        .execute(
            &crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery {
                social_user_id,
            },
        )
        .await;

    let mut rows: Vec<Vec<InlineKeyboardButton>> = vec![vec![InlineKeyboardButton::callback(
        TelegramBotDigestRepositoryAction::All.label(),
        TelegramBotDigestRepositoryAction::All.to_callback_data(),
    )]];

    if let Ok(result) = bound_repos {
        for repo in &result.repositories {
            rows.push(vec![InlineKeyboardButton::callback(
                format!("{}/{}", repo.owner, repo.name),
                repo.id.0.to_string(),
            )]);
        }
    }

    rows.push(vec![InlineKeyboardButton::callback(
        TelegramBotDigestRepositoryAction::Cancel.label(),
        TelegramBotDigestRepositoryAction::Cancel.to_callback_data(),
    )]);

    let digest_type = match DigestType::from_str(data) {
        Some(t) => t,
        None => return Ok(()),
    };

    dialogue
        .update(TelegramBotDialogueState::Digest(
            TelegramBotDigestState::ChooseRepository { digest_type },
        ))
        .await?;

    bot.send_message(
        msg.chat().id,
        t!("telegram_bot.dialogues.digest.choose_repository").to_string(),
    )
    .reply_markup(InlineKeyboardMarkup::new(rows))
    .await?;

    Ok(())
}

async fn handle_choose_repository(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    query: CallbackQuery,
    digest_type: DigestType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let repository_id = match TelegramBotDigestRepositoryAction::from_callback_data(data) {
        Ok(TelegramBotDigestRepositoryAction::Cancel) => {
            bot.send_message(
                msg.chat().id,
                t!("telegram_bot.common.cancelled").to_string(),
            )
            .await?;

            dialogue.exit().await.ok();
            return Ok(());
        }
        Ok(TelegramBotDigestRepositoryAction::All) => None,
        Err(_) => match data.parse::<i32>() {
            Ok(id) => Some(id),
            Err(_) => return Ok(()),
        },
    };

    dialogue
        .update(TelegramBotDialogueState::Digest(
            TelegramBotDigestState::ChooseTime {
                digest_type,
                repository_id,
            },
        ))
        .await?;

    bot.send_message(
        msg.chat().id,
        t!("telegram_bot.dialogues.digest.enter_time").to_string(),
    )
    .await?;

    Ok(())
}

async fn handle_choose_time(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    msg: Message,
    (digest_type, repository_id): (DigestType, Option<i32>),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or("").trim();

    let parts: Vec<&str> = text.split(':').collect();

    if parts.len() != 2 {
        bot.send_message(
            msg.chat.id,
            t!("telegram_bot.dialogues.digest.invalid_time").to_string(),
        )
        .await?;

        return Ok(());
    }

    let hour: i8 = match parts[0].parse() {
        Ok(h) if (0..24).contains(&h) => h,
        _ => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.digest.invalid_time").to_string(),
            )
            .await?;

            return Ok(());
        }
    };

    let minute: i8 = match parts[1].parse() {
        Ok(m) if (0..60).contains(&m) => m,
        _ => {
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.digest.invalid_time").to_string(),
            )
            .await?;

            return Ok(());
        }
    };

    let social_user_id = SocialUserId(msg.from.as_ref().map(|u| u.id.0 as i32).unwrap_or(0));

    let day_of_week = if digest_type == DigestType::Weekly {
        Some(1) // Monday
    } else {
        None
    };

    let cmd = CreateDigestSubscriptionCommand {
        social_user_id,
        repository_id: repository_id.map(RepositoryId),
        digest_type,
        send_hour: hour,
        send_minute: minute,
        day_of_week,
    };

    let reply = match executors
        .commands
        .create_digest_subscription
        .execute(&cmd)
        .await
    {
        Ok(_) => t!("telegram_bot.dialogues.digest.created").to_string(),

        Err(e) => {
            tracing::error!(error = %e, "Failed to create digest subscription");
            t!("telegram_bot.dialogues.digest.error").to_string()
        }
    };

    bot.send_message(msg.chat.id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}

async fn handle_confirm_delete(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    query: CallbackQuery,
    subscription_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = query.data.as_deref().unwrap_or("");

    let msg = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };

    let is_confirmed = TelegramBotConfirmAction::from_callback_data(data)
        .map(|a| matches!(a, TelegramBotConfirmAction::Yes))
        .unwrap_or(false);

    if !is_confirmed {
        bot.send_message(
            msg.chat().id,
            t!("telegram_bot.common.cancelled").to_string(),
        )
        .await?;

        dialogue.exit().await.ok();
        return Ok(());
    }

    let cmd = DeleteDigestSubscriptionCommand {
        id: DigestSubscriptionId(subscription_id),
    };

    let reply = match executors
        .commands
        .delete_digest_subscription
        .execute(&cmd)
        .await
    {
        Ok(_) => t!("telegram_bot.dialogues.digest.deleted").to_string(),

        Err(e) => {
            tracing::error!(error = %e, "Failed to delete digest subscription");
            t!("telegram_bot.dialogues.digest.error").to_string()
        }
    };

    bot.send_message(msg.chat().id, reply).await?;

    dialogue.exit().await.ok();

    Ok(())
}
