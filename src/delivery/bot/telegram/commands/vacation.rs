use crate::application::user_preferences::commands::update_user_preferences::command::{
    UpdateUserPreferencesExecutorCommand, UserPreferencesPatch,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use chrono::{Duration, Utc};
use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, ParseMode};

pub struct TelegramBotVacationCommandHandler {
    bot: Bot,
    msg: Message,
    executors: Arc<ApplicationBoostrapExecutors>,
    raw_arg: String,
    social_user_id: SocialUserId,
}

impl TelegramBotVacationCommandHandler {
    pub fn new(
        bot: Bot,
        msg: Message,
        executors: Arc<ApplicationBoostrapExecutors>,
        raw_arg: String,
        social_user_id: SocialUserId,
    ) -> Self {
        Self {
            bot,
            msg,
            executors,
            raw_arg,
            social_user_id,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let arg = self.raw_arg.trim().to_lowercase();

        let (patch, reply) = if arg == "off" || arg == "0" {
            (
                UserPreferencesPatch::ClearVacation,
                t!("telegram_bot.commands.vacation.cleared").to_string(),
            )
        } else if let Some(days) = parse_days(&arg) {
            let until = Utc::now() + Duration::days(days as i64);
            let until_local = until.format("%d.%m.%Y").to_string();
            (
                UserPreferencesPatch::SetVacation { until },
                t!(
                    "telegram_bot.commands.vacation.set",
                    days = days,
                    until = until_local
                )
                .to_string(),
            )
        } else {
            self.bot
                .send_message(
                    self.msg.chat.id,
                    t!("telegram_bot.commands.vacation.usage").to_string(),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            return Ok(());
        };

        match self
            .executors
            .commands
            .update_user_preferences
            .execute(&UpdateUserPreferencesExecutorCommand {
                social_user_id: self.social_user_id,
                patch,
            })
            .await
        {
            Ok(_) => {
                self.bot
                    .send_message(self.msg.chat.id, reply)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to apply /vacation patch");
                self.bot
                    .send_message(
                        self.msg.chat.id,
                        t!("telegram_bot.commands.vacation.error").to_string(),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }

        Ok(())
    }
}

fn parse_days(arg: &str) -> Option<u32> {
    let trimmed = arg.trim_end_matches(|c: char| c == 'd' || c == 'D' || c.is_whitespace());
    let n: u32 = trimmed.parse().ok()?;
    if n == 0 || n > 365 { None } else { Some(n) }
}
