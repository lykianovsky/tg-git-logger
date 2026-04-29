use crate::application::user_preferences::queries::get_user_preferences::query::GetUserPreferencesQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::dialogues::helpers::edit_menu;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::notifications_events::TelegramBotNotificationsEventAction;
use crate::delivery::bot::telegram::keyboards::actions::notifications_menu::TelegramBotNotificationsMenuAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user_preferences::entities::user_preferences::UserPreferences;
use crate::domain::user_preferences::value_objects::notification_event_kind::NotificationEventKind;
use crate::utils::builder::message::MessageBuilder;
use chrono::Utc;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode};

pub async fn load_prefs(
    executors: &Arc<ApplicationBoostrapExecutors>,
    social_user_id: SocialUserId,
) -> Option<UserPreferences> {
    match executors
        .queries
        .get_user_preferences
        .execute(&GetUserPreferencesQuery { social_user_id })
        .await
    {
        Ok(r) => r.preferences,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load user preferences");
            None
        }
    }
}

fn build_main_menu(
    prefs: Option<&UserPreferences>,
    config: &Arc<ApplicationConfig>,
) -> (String, InlineKeyboardMarkup) {
    let text = format_prefs_text(prefs, config);

    let kb = KeyboardBuilder::new()
        .row::<TelegramBotNotificationsMenuAction>(vec![
            TelegramBotNotificationsMenuAction::DndWindow,
            TelegramBotNotificationsMenuAction::Snooze,
        ])
        .row::<TelegramBotNotificationsMenuAction>(vec![
            TelegramBotNotificationsMenuAction::Vacation,
            TelegramBotNotificationsMenuAction::Events,
        ])
        .row::<TelegramBotNotificationsMenuAction>(vec![
            TelegramBotNotificationsMenuAction::PriorityOnly,
            TelegramBotNotificationsMenuAction::Reset,
        ])
        .row::<TelegramBotNotificationsMenuAction>(vec![TelegramBotNotificationsMenuAction::Cancel])
        .build();

    (text, kb)
}

pub async fn send_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    _shared: &Arc<ApplicationSharedDependency>,
    config: &Arc<ApplicationConfig>,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prefs = load_prefs(executors, social_user_id).await;
    let (text, kb) = build_main_menu(prefs.as_ref(), config);

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(kb)
        .await?;

    Ok(())
}

pub async fn edit_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    config: &Arc<ApplicationConfig>,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prefs = load_prefs(executors, social_user_id).await;
    let (text, kb) = build_main_menu(prefs.as_ref(), config);
    edit_menu(bot, chat_id, message_id, &text, Some(kb)).await
}

fn build_events_menu(enabled: &[NotificationEventKind]) -> InlineKeyboardMarkup {
    let label = |action: &TelegramBotNotificationsEventAction, kind: NotificationEventKind| {
        let mark = if enabled.contains(&kind) {
            "✅"
        } else {
            "⬜"
        };
        format!("{} {}", mark, action.label())
    };

    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for (action, kind) in [
        (
            TelegramBotNotificationsEventAction::Pr,
            NotificationEventKind::Pr,
        ),
        (
            TelegramBotNotificationsEventAction::Review,
            NotificationEventKind::Review,
        ),
        (
            TelegramBotNotificationsEventAction::Comment,
            NotificationEventKind::Comment,
        ),
        (
            TelegramBotNotificationsEventAction::Ci,
            NotificationEventKind::Ci,
        ),
        (
            TelegramBotNotificationsEventAction::Release,
            NotificationEventKind::Release,
        ),
    ] {
        rows.push(vec![InlineKeyboardButton::callback(
            label(&action, kind),
            action.to_callback_data().to_string(),
        )]);
    }
    rows.push(vec![InlineKeyboardButton::callback(
        TelegramBotNotificationsEventAction::Back
            .label()
            .to_string(),
        TelegramBotNotificationsEventAction::Back
            .to_callback_data()
            .to_string(),
    )]);

    InlineKeyboardMarkup::new(rows)
}

pub async fn edit_events_menu(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prefs = load_prefs(executors, social_user_id).await;
    let enabled = prefs
        .as_ref()
        .map(|p| p.enabled_events.clone())
        .unwrap_or_else(NotificationEventKind::all_default_enabled);
    let kb = build_events_menu(&enabled);
    let text = t!("telegram_bot.dialogues.notifications.events_title").to_string();
    edit_menu(bot, chat_id, message_id, &text, Some(kb)).await
}

pub async fn send_events_menu(
    bot: &Bot,
    chat_id: ChatId,
    executors: &Arc<ApplicationBoostrapExecutors>,
    _config: &Arc<ApplicationConfig>,
    social_user_id: SocialUserId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let prefs = load_prefs(executors, social_user_id).await;
    let enabled = prefs
        .as_ref()
        .map(|p| p.enabled_events.clone())
        .unwrap_or_else(NotificationEventKind::all_default_enabled);
    let rows = build_events_menu(&enabled);

    bot.send_message(
        chat_id,
        t!("telegram_bot.dialogues.notifications.events_title").to_string(),
    )
    .reply_markup(rows)
    .await?;

    Ok(())
}

fn format_prefs_text(prefs: Option<&UserPreferences>, config: &Arc<ApplicationConfig>) -> String {
    let mut b = MessageBuilder::new();
    b = b.bold(&t!("telegram_bot.dialogues.notifications.title").to_string());
    b = b.empty_line();

    let dnd_label = match prefs.and_then(|p| p.dnd_window) {
        Some(w) => format!("{}–{}", w.start.format("%H:%M"), w.end.format("%H:%M")),
        None => format!(
            "{}–{} ({})",
            config.notifications.default_dnd_start.format("%H:%M"),
            config.notifications.default_dnd_end.format("%H:%M"),
            t!("telegram_bot.dialogues.notifications.default")
        ),
    };
    b = b.section(
        &t!("telegram_bot.dialogues.notifications.dnd").to_string(),
        &dnd_label,
    );

    let tz_label = match prefs.and_then(|p| p.timezone) {
        Some(tz) => tz.name().to_string(),
        None => format!(
            "{} ({})",
            config.notifications.default_timezone.name(),
            t!("telegram_bot.dialogues.notifications.default")
        ),
    };
    b = b.section(
        &t!("telegram_bot.dialogues.notifications.timezone").to_string(),
        &tz_label,
    );

    let now = Utc::now();
    let snooze_label = match prefs.and_then(|p| p.snooze_until) {
        Some(s) if s > now => format!("до {}", s.format("%d.%m %H:%M")),
        _ => t!("telegram_bot.dialogues.notifications.off").to_string(),
    };
    b = b.section(
        &t!("telegram_bot.dialogues.notifications.snooze").to_string(),
        &snooze_label,
    );

    let vacation_label = match prefs.and_then(|p| p.vacation_until) {
        Some(v) if v > now => format!("до {}", v.format("%d.%m.%Y")),
        _ => t!("telegram_bot.dialogues.notifications.off").to_string(),
    };
    b = b.section(
        &t!("telegram_bot.dialogues.notifications.vacation").to_string(),
        &vacation_label,
    );

    let events: Vec<String> = prefs
        .map(|p| p.enabled_events.clone())
        .unwrap_or_else(NotificationEventKind::all_default_enabled)
        .iter()
        .map(|e| event_display_label(e).to_string())
        .collect();
    let events_label = if events.is_empty() {
        t!("telegram_bot.dialogues.notifications.off").to_string()
    } else {
        events.join(", ")
    };
    b = b.section(
        &t!("telegram_bot.dialogues.notifications.events").to_string(),
        &events_label,
    );

    let priority_label = if prefs.map(|p| p.priority_only).unwrap_or(false) {
        "✅"
    } else {
        "⬜"
    };
    b = b.section(
        &t!("telegram_bot.dialogues.notifications.priority_only").to_string(),
        priority_label,
    );

    b.build()
}

fn event_display_label(e: &NotificationEventKind) -> &'static str {
    match e {
        NotificationEventKind::Pr => "PR",
        NotificationEventKind::Review => "Ревью",
        NotificationEventKind::Comment => "Комментарии",
        NotificationEventKind::Ci => "CI",
        NotificationEventKind::Release => "Релизы",
    }
}
