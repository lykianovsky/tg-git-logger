use crate::application::user::queries::get_user_overview::error::GetUserOverviewError;
use crate::application::user::queries::get_user_overview::query::GetUserOverviewQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use chrono::Utc;
use chrono_tz::Europe::Moscow;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::ParseMode;

pub struct TelegramBotWhoamiCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
}

impl TelegramBotWhoamiCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executors: Arc<ApplicationBoostrapExecutors>,
    ) -> Self {
        Self { context, executors }
    }

    #[tracing::instrument(name = "tg.whoami", skip_all, fields(user_id = self.context.user.id.0))]
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let overview = match self
            .executors
            .queries
            .get_user_overview
            .execute(&GetUserOverviewQuery { social_user_id })
            .await
        {
            Ok(o) => o,
            Err(GetUserOverviewError::UserNotFound) => {
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.whoami.not_registered").to_string(),
                    )
                    .await?;
                return Ok(());
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to load user overview");
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.whoami.error").to_string(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let github_text = overview
            .github_login
            .clone()
            .unwrap_or_else(|| "—".to_string());

        let roles_text = if overview.roles.is_empty() {
            "—".to_string()
        } else {
            overview
                .roles
                .iter()
                .map(role_label)
                .collect::<Vec<_>>()
                .join(", ")
        };

        let dnd_text = match &overview.dnd_window {
            Some(w) => format!(
                "{}–{} МСК",
                w.start.format("%H:%M"),
                w.end.format("%H:%M")
            ),
            None => t!("telegram_bot.commands.whoami.default").to_string(),
        };

        let timezone_text = match overview.timezone {
            Some(tz) => tz.name().to_string(),
            None => t!("telegram_bot.commands.whoami.default").to_string(),
        };

        let now = Utc::now();
        let vacation_text = match overview.vacation_until {
            Some(until) if until > now => until
                .with_timezone(&Moscow)
                .format("%d.%m.%Y")
                .to_string(),
            _ => t!("telegram_bot.commands.whoami.off").to_string(),
        };

        let snooze_text = match overview.snooze_until {
            Some(until) if until > now => until
                .with_timezone(&Moscow)
                .format("%d.%m %H:%M МСК")
                .to_string(),
            _ => t!("telegram_bot.commands.whoami.off").to_string(),
        };

        let repos_text = if overview.repositories.is_empty() {
            t!("telegram_bot.commands.whoami.no_repos").to_string()
        } else {
            overview
                .repositories
                .iter()
                .map(|r| format!("{}/{}", r.owner, r.name))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let text = MessageBuilder::new()
            .bold(&t!("telegram_bot.commands.whoami.title").to_string())
            .empty_line()
            .with_html_escape(true)
            .section(
                &t!("telegram_bot.commands.whoami.github").to_string(),
                &github_text,
            )
            .section(
                &t!("telegram_bot.commands.whoami.roles").to_string(),
                &roles_text,
            )
            .section(
                &t!("telegram_bot.commands.whoami.dnd").to_string(),
                &dnd_text,
            )
            .section(
                &t!("telegram_bot.commands.whoami.timezone").to_string(),
                &timezone_text,
            )
            .section(
                &t!("telegram_bot.commands.whoami.vacation").to_string(),
                &vacation_text,
            )
            .section(
                &t!("telegram_bot.commands.whoami.snooze").to_string(),
                &snooze_text,
            )
            .section(
                &t!("telegram_bot.commands.whoami.repositories").to_string(),
                &repos_text,
            )
            .build();

        self.context
            .bot
            .send_message(self.context.msg.chat.id, text)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
}

fn role_label(role: &RoleName) -> String {
    match role {
        RoleName::Admin => "Admin".to_string(),
        RoleName::Developer => "Developer".to_string(),
        RoleName::QualityAssurance => "QA".to_string(),
        RoleName::ProductManager => "PM".to_string(),
    }
}
