use crate::application::health_ping::queries::get_all_health_pings::query::GetAllHealthPingsQuery;
use crate::application::user::queries::get_user_roles_by_telegram_id::error::GetUserRolesByTelegramIdError;
use crate::application::user::queries::get_user_roles_by_telegram_id::query::GetUserRolesByTelegramIdQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use chrono_tz::Europe::Moscow;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::ParseMode;

pub struct TelegramBotStatusCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
}

impl TelegramBotStatusCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executors: Arc<ApplicationBoostrapExecutors>,
    ) -> Self {
        Self { context, executors }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let roles = match self
            .executors
            .queries
            .get_user_roles_by_telegram_id
            .execute(&GetUserRolesByTelegramIdQuery { social_user_id })
            .await
        {
            Ok(r) => r.roles,
            Err(GetUserRolesByTelegramIdError::UserNotFound) => {
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.status.not_registered").to_string(),
                    )
                    .await?;
                return Ok(());
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to load user roles for /status");
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.status.error").to_string(),
                    )
                    .await?;
                return Ok(());
            }
        };

        if !roles.contains(&RoleName::Admin) {
            self.context
                .bot
                .send_message(
                    self.context.msg.chat.id,
                    t!("telegram_bot.commands.status.forbidden").to_string(),
                )
                .await?;
            return Ok(());
        }

        let pings = match self
            .executors
            .queries
            .get_all_health_pings
            .execute(&GetAllHealthPingsQuery)
            .await
        {
            Ok(r) => r.pings,
            Err(e) => {
                tracing::error!(error = %e, "Failed to load health pings for /status");
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.status.error").to_string(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let mut builder = MessageBuilder::new()
            .bold(&t!("telegram_bot.commands.status.title").to_string())
            .empty_line();

        if pings.is_empty() {
            builder = builder.line(&t!("telegram_bot.commands.status.empty").to_string());
        } else {
            for ping in &pings {
                let icon = if !ping.is_active {
                    "⏸"
                } else {
                    match ping.last_status.as_deref() {
                        Some("ok") => "🟢",
                        Some("error") => "🔴",
                        _ => "⚪",
                    }
                };

                let latency = ping
                    .last_response_ms
                    .map(|ms| format!("{}ms", ms))
                    .unwrap_or_else(|| "—".to_string());

                let checked = ping
                    .last_checked_at
                    .map(|t| t.with_timezone(&Moscow).format("%H:%M МСК").to_string())
                    .unwrap_or_else(|| "—".to_string());

                builder = builder.with_html_escape(false).raw(&format!(
                    "{} <b>{}</b> — {} ({}, {})\n",
                    icon,
                    MessageBuilder::escape_html(&ping.name),
                    MessageBuilder::escape_html(&ping.url),
                    latency,
                    checked,
                ));

                if let Some(err) = &ping.last_error_message {
                    builder = builder.with_html_escape(true).line(&format!(
                        "    └ {}",
                        if err.len() > 120 { &err[..120] } else { err }
                    ));
                }
            }
        }

        let text = builder.build();

        self.context
            .bot
            .send_message(self.context.msg.chat.id, text)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
}
