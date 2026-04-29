use crate::application::user_preferences::commands::update_user_preferences::command::{
    UpdateUserPreferencesExecutorCommand, UserPreferencesPatch,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::dialogues::helpers::{close_menu, edit_menu};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::confirm::TelegramBotConfirmAction;
use crate::delivery::bot::telegram::keyboards::actions::notifications_events::TelegramBotNotificationsEventAction;
use crate::delivery::bot::telegram::keyboards::actions::notifications_menu::TelegramBotNotificationsMenuAction;
use crate::delivery::bot::telegram::keyboards::actions::notifications_snooze::TelegramBotNotificationsSnoozeAction;
use crate::delivery::bot::telegram::keyboards::actions::notifications_vacation::TelegramBotNotificationsVacationAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user_preferences::value_objects::notification_event_kind::NotificationEventKind;
use chrono::{Duration, NaiveTime, Utc};
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::{Bot, dptree};

mod render;

pub use render::send_main_menu;

#[derive(Debug, Clone, Default)]
pub enum TelegramBotNotificationsState {
    #[default]
    Menu,
    EditDndWindow,
    ChooseSnooze,
    ChooseVacation,
    EditEvents,
    ConfirmReset,
}

pub struct TelegramBotNotificationsDispatcher {}

impl TelegramBotNotificationsDispatcher {
    pub fn new()
    -> Handler<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query()
            .branch(case![TelegramBotNotificationsState::Menu].endpoint(handle_menu_action))
            .branch(case![TelegramBotNotificationsState::ChooseSnooze].endpoint(handle_snooze))
            .branch(case![TelegramBotNotificationsState::ChooseVacation].endpoint(handle_vacation))
            .branch(case![TelegramBotNotificationsState::EditEvents].endpoint(handle_events))
            .branch(
                case![TelegramBotNotificationsState::ConfirmReset].endpoint(handle_confirm_reset),
            );

        let messages = Update::filter_message()
            .branch(case![TelegramBotNotificationsState::EditDndWindow].endpoint(handle_dnd_input));

        dptree::entry().branch(queries).branch(messages)
    }
}

async fn handle_menu_action(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    shared: Arc<ApplicationSharedDependency>,
    config: Arc<ApplicationConfig>,
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

    let action = match TelegramBotNotificationsMenuAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);

    match action {
        TelegramBotNotificationsMenuAction::DndWindow => {
            dialogue
                .update(TelegramBotDialogueState::Notifications(
                    TelegramBotNotificationsState::EditDndWindow,
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.notifications.enter_dnd").to_string(),
                None,
            )
            .await?;
        }
        TelegramBotNotificationsMenuAction::Snooze => {
            let kb = KeyboardBuilder::new()
                .row::<TelegramBotNotificationsSnoozeAction>(vec![
                    TelegramBotNotificationsSnoozeAction::TwoHours,
                    TelegramBotNotificationsSnoozeAction::FourHours,
                ])
                .row::<TelegramBotNotificationsSnoozeAction>(vec![
                    TelegramBotNotificationsSnoozeAction::UntilMorning,
                ])
                .row::<TelegramBotNotificationsSnoozeAction>(vec![
                    TelegramBotNotificationsSnoozeAction::Clear,
                    TelegramBotNotificationsSnoozeAction::Back,
                ])
                .build();
            dialogue
                .update(TelegramBotDialogueState::Notifications(
                    TelegramBotNotificationsState::ChooseSnooze,
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.notifications.choose_snooze").to_string(),
                Some(kb),
            )
            .await?;
        }
        TelegramBotNotificationsMenuAction::Vacation => {
            let kb = KeyboardBuilder::new()
                .row::<TelegramBotNotificationsVacationAction>(vec![
                    TelegramBotNotificationsVacationAction::OneDay,
                    TelegramBotNotificationsVacationAction::ThreeDays,
                    TelegramBotNotificationsVacationAction::SevenDays,
                ])
                .row::<TelegramBotNotificationsVacationAction>(vec![
                    TelegramBotNotificationsVacationAction::Clear,
                    TelegramBotNotificationsVacationAction::Back,
                ])
                .build();
            dialogue
                .update(TelegramBotDialogueState::Notifications(
                    TelegramBotNotificationsState::ChooseVacation,
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.notifications.choose_vacation").to_string(),
                Some(kb),
            )
            .await?;
        }
        TelegramBotNotificationsMenuAction::Events => {
            dialogue
                .update(TelegramBotDialogueState::Notifications(
                    TelegramBotNotificationsState::EditEvents,
                ))
                .await?;
            render::edit_events_menu(&bot, chat_id, message_id, &executors, social_user_id).await?;
        }
        TelegramBotNotificationsMenuAction::PriorityOnly => {
            let prefs = render::load_prefs(&executors, social_user_id).await;
            let new_value = !prefs.as_ref().map(|p| p.priority_only).unwrap_or(false);
            apply_patch(
                &executors,
                social_user_id,
                UserPreferencesPatch::SetPriorityOnly { enabled: new_value },
            )
            .await;
            render::edit_main_menu(
                &bot,
                chat_id,
                message_id,
                &executors,
                &config,
                social_user_id,
            )
            .await?;
        }
        TelegramBotNotificationsMenuAction::Reset => {
            let kb = KeyboardBuilder::new()
                .row::<TelegramBotConfirmAction>(vec![
                    TelegramBotConfirmAction::Yes,
                    TelegramBotConfirmAction::No,
                ])
                .build();
            dialogue
                .update(TelegramBotDialogueState::Notifications(
                    TelegramBotNotificationsState::ConfirmReset,
                ))
                .await?;
            edit_menu(
                &bot,
                chat_id,
                message_id,
                &t!("telegram_bot.dialogues.notifications.confirm_reset").to_string(),
                Some(kb),
            )
            .await?;
        }
        TelegramBotNotificationsMenuAction::Cancel => {
            close_menu(&bot, chat_id, message_id).await;
            dialogue.exit().await.ok();
        }
    }

    let _ = shared;
    Ok(())
}

async fn handle_dnd_input(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    shared: Arc<ApplicationSharedDependency>,
    config: Arc<ApplicationConfig>,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or("").trim();

    let parts: Vec<&str> = text.split('-').map(|s| s.trim()).collect();
    let invalid = || async {
        teloxide::payloads::SendMessageSetters::parse_mode(
            bot.send_message(
                msg.chat.id,
                t!("telegram_bot.dialogues.notifications.invalid_dnd").to_string(),
            ),
            teloxide::types::ParseMode::Html,
        )
        .await
    };

    if parts.len() != 2 {
        invalid().await?;
        return Ok(());
    }

    let start = match NaiveTime::parse_from_str(parts[0], "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            invalid().await?;
            return Ok(());
        }
    };
    let end = match NaiveTime::parse_from_str(parts[1], "%H:%M") {
        Ok(t) => t,
        Err(_) => {
            invalid().await?;
            return Ok(());
        }
    };

    let social_user_id = SocialUserId(msg.from.as_ref().map(|u| u.id.0 as i32).unwrap_or(0));

    apply_patch(
        &executors,
        social_user_id,
        UserPreferencesPatch::SetDndWindow { start, end },
    )
    .await;

    // После text-input всегда send новое (старое может быть скрыто за вводом юзера)
    send_main_menu(
        &bot,
        msg.chat.id,
        &executors,
        &shared,
        &config,
        social_user_id,
    )
    .await?;
    dialogue
        .update(TelegramBotDialogueState::Notifications(
            TelegramBotNotificationsState::Menu,
        ))
        .await?;

    Ok(())
}

async fn handle_snooze(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    shared: Arc<ApplicationSharedDependency>,
    config: Arc<ApplicationConfig>,
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
    let action = match TelegramBotNotificationsSnoozeAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);
    let now = Utc::now();

    let patch = match action {
        TelegramBotNotificationsSnoozeAction::TwoHours => Some(UserPreferencesPatch::SetSnooze {
            until: now + Duration::hours(2),
        }),
        TelegramBotNotificationsSnoozeAction::FourHours => Some(UserPreferencesPatch::SetSnooze {
            until: now + Duration::hours(4),
        }),
        TelegramBotNotificationsSnoozeAction::UntilMorning => {
            let prefs = render::load_prefs(&executors, social_user_id).await;
            let until = shared
                .quiet_hours_resolver
                .next_active_at(prefs.as_ref(), now);
            Some(UserPreferencesPatch::SetSnooze { until })
        }
        TelegramBotNotificationsSnoozeAction::Clear => Some(UserPreferencesPatch::ClearSnooze),
        TelegramBotNotificationsSnoozeAction::Back => None,
    };

    if let Some(p) = patch {
        apply_patch(&executors, social_user_id, p).await;
    }

    render::edit_main_menu(
        &bot,
        chat_id,
        message_id,
        &executors,
        &config,
        social_user_id,
    )
    .await?;
    dialogue
        .update(TelegramBotDialogueState::Notifications(
            TelegramBotNotificationsState::Menu,
        ))
        .await?;

    Ok(())
}

async fn handle_vacation(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    shared: Arc<ApplicationSharedDependency>,
    config: Arc<ApplicationConfig>,
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
    let action = match TelegramBotNotificationsVacationAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);
    let now = Utc::now();

    let patch = match action {
        TelegramBotNotificationsVacationAction::OneDay => Some(UserPreferencesPatch::SetVacation {
            until: now + Duration::days(1),
        }),
        TelegramBotNotificationsVacationAction::ThreeDays => {
            Some(UserPreferencesPatch::SetVacation {
                until: now + Duration::days(3),
            })
        }
        TelegramBotNotificationsVacationAction::SevenDays => {
            Some(UserPreferencesPatch::SetVacation {
                until: now + Duration::days(7),
            })
        }
        TelegramBotNotificationsVacationAction::Clear => Some(UserPreferencesPatch::ClearVacation),
        TelegramBotNotificationsVacationAction::Back => None,
    };

    if let Some(p) = patch {
        apply_patch(&executors, social_user_id, p).await;
    }

    render::edit_main_menu(
        &bot,
        chat_id,
        message_id,
        &executors,
        &config,
        social_user_id,
    )
    .await?;
    dialogue
        .update(TelegramBotDialogueState::Notifications(
            TelegramBotNotificationsState::Menu,
        ))
        .await?;

    let _ = shared;
    Ok(())
}

async fn handle_events(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
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
    let action = match TelegramBotNotificationsEventAction::from_callback_data(data) {
        Ok(a) => a,
        Err(_) => return Ok(()),
    };

    let social_user_id = SocialUserId(query.from.id.0 as i32);

    let event_kind = match action {
        TelegramBotNotificationsEventAction::Pr => Some(NotificationEventKind::Pr),
        TelegramBotNotificationsEventAction::Review => Some(NotificationEventKind::Review),
        TelegramBotNotificationsEventAction::Comment => Some(NotificationEventKind::Comment),
        TelegramBotNotificationsEventAction::Ci => Some(NotificationEventKind::Ci),
        TelegramBotNotificationsEventAction::Release => Some(NotificationEventKind::Release),
        TelegramBotNotificationsEventAction::Back => None,
    };

    if let Some(event) = event_kind {
        let prefs = render::load_prefs(&executors, social_user_id).await;
        let was_enabled = prefs
            .as_ref()
            .map(|p| p.enabled_events.contains(&event))
            .unwrap_or(true);
        apply_patch(
            &executors,
            social_user_id,
            UserPreferencesPatch::ToggleEnabledEvent {
                event,
                enabled: !was_enabled,
            },
        )
        .await;
        render::edit_events_menu(&bot, chat_id, message_id, &executors, social_user_id).await?;
    } else {
        render::edit_main_menu(
            &bot,
            chat_id,
            message_id,
            &executors,
            &config,
            social_user_id,
        )
        .await?;
        dialogue
            .update(TelegramBotDialogueState::Notifications(
                TelegramBotNotificationsState::Menu,
            ))
            .await?;
    }

    Ok(())
}

async fn handle_confirm_reset(
    bot: Bot,
    dialogue: TelegramBotDialogueType,
    executors: Arc<ApplicationBoostrapExecutors>,
    shared: Arc<ApplicationSharedDependency>,
    config: Arc<ApplicationConfig>,
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

    let social_user_id = SocialUserId(query.from.id.0 as i32);

    let confirmed = TelegramBotConfirmAction::from_callback_data(data)
        .map(|a| matches!(a, TelegramBotConfirmAction::Yes))
        .unwrap_or(false);

    if confirmed {
        apply_patch(&executors, social_user_id, UserPreferencesPatch::Reset).await;
    }

    render::edit_main_menu(
        &bot,
        chat_id,
        message_id,
        &executors,
        &config,
        social_user_id,
    )
    .await?;
    dialogue
        .update(TelegramBotDialogueState::Notifications(
            TelegramBotNotificationsState::Menu,
        ))
        .await?;

    let _ = shared;
    Ok(())
}

async fn apply_patch(
    executors: &Arc<ApplicationBoostrapExecutors>,
    social_user_id: SocialUserId,
    patch: UserPreferencesPatch,
) {
    let cmd = UpdateUserPreferencesExecutorCommand {
        social_user_id,
        patch,
    };
    if let Err(e) = executors
        .commands
        .update_user_preferences
        .execute(&cmd)
        .await
    {
        tracing::error!(error = %e, "Failed to update user preferences");
    }
}
