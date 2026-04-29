use crate::application::digest::queries::get_user_digest_subscriptions::query::GetUserDigestSubscriptionsQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::digest::TelegramBotDigestState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::actions::digest_list::TelegramBotDigestListAction;
use crate::domain::digest::value_objects::digest_type::DigestType;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub struct TelegramBotDigestCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotDigestCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executors: Arc<ApplicationBoostrapExecutors>,
        dialogue: Arc<TelegramBotDialogueType>,
    ) -> Self {
        Self {
            context,
            executors,
            dialogue,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let query = GetUserDigestSubscriptionsQuery { social_user_id };

        let subscriptions = match self
            .executors
            .queries
            .get_user_digest_subscriptions
            .execute(&query)
            .await
        {
            Ok(result) => result.subscriptions,
            Err(e) => {
                tracing::error!(error = %e, "Failed to get digest subscriptions");

                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.dialogues.digest.error").to_string(),
                    )
                    .await?;

                return Ok(());
            }
        };

        let mut builder = MessageBuilder::new();
        builder = builder.bold(&t!("telegram_bot.commands.digest.title").to_string());
        builder = builder.empty_line();

        if subscriptions.is_empty() {
            builder = builder.line(&t!("telegram_bot.commands.digest.empty").to_string());
        } else {
            for (i, sub) in subscriptions.iter().enumerate() {
                let type_label = match sub.digest_type {
                    DigestType::Daily => t!("telegram_bot.dialogues.digest.type_daily").to_string(),
                    DigestType::Weekly => {
                        t!("telegram_bot.dialogues.digest.type_weekly").to_string()
                    }
                };

                let status = if sub.is_active { "✅" } else { "⏸" };

                let repo_label = match &sub.repository_id {
                    Some(_) => t!("telegram_bot.dialogues.digest.specific_repo").to_string(),
                    None => t!("telegram_bot.dialogues.digest.all_repositories").to_string(),
                };

                let line = format!(
                    "{}. {} {} — {} — {:02}:{:02}",
                    i + 1,
                    status,
                    type_label,
                    repo_label,
                    sub.send_hour,
                    sub.send_minute,
                );

                builder = builder.line(&line);
            }
        }

        let text = builder.build();

        let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        for sub in &subscriptions {
            let toggle_label = if sub.is_active {
                format!(
                    "⏸ {} #{}",
                    t!("telegram_bot.dialogues.digest.btn_disable").to_string(),
                    sub.id.0
                )
            } else {
                format!(
                    "▶️ {} #{}",
                    t!("telegram_bot.dialogues.digest.btn_enable").to_string(),
                    sub.id.0
                )
            };

            rows.push(vec![
                InlineKeyboardButton::callback(
                    toggle_label,
                    TelegramBotDigestListAction::toggle_callback(sub.id.0),
                ),
                InlineKeyboardButton::callback(
                    format!("🗑 #{}", sub.id.0),
                    TelegramBotDigestListAction::delete_callback(sub.id.0),
                ),
            ]);
        }

        rows.push(vec![InlineKeyboardButton::callback(
            t!("telegram_bot.dialogues.digest.btn_create").to_string(),
            TelegramBotDigestListAction::Create.to_callback_data(),
        )]);

        rows.push(vec![InlineKeyboardButton::callback(
            t!("telegram_bot.common.cancel").to_string(),
            TelegramBotDigestListAction::Cancel.to_callback_data(),
        )]);

        let keyboard = InlineKeyboardMarkup::new(rows);

        self.dialogue
            .update(TelegramBotDialogueState::Digest(
                TelegramBotDigestState::List,
            ))
            .await?;

        self.context
            .bot
            .send_message(self.context.msg.chat.id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
