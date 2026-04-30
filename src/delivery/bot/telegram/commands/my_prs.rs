use crate::application::user::queries::get_my_pull_requests::error::GetMyPullRequestsError;
use crate::application::user::queries::get_my_pull_requests::query::GetMyPullRequestsQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use chrono::Utc;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::ParseMode;

pub struct TelegramBotMyPrsCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
}

impl TelegramBotMyPrsCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executors: Arc<ApplicationBoostrapExecutors>,
    ) -> Self {
        Self { context, executors }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let social_user_id = SocialUserId(self.context.user.id.0 as i32);

        let prs = match self
            .executors
            .queries
            .get_my_pull_requests
            .execute(&GetMyPullRequestsQuery { social_user_id })
            .await
        {
            Ok(r) => r.prs,
            Err(GetMyPullRequestsError::UserNotFound) => {
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.my_prs.not_registered").to_string(),
                    )
                    .await?;
                return Ok(());
            }
            Err(GetMyPullRequestsError::NoGithubAccount) => {
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.my_prs.no_github").to_string(),
                    )
                    .await?;
                return Ok(());
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to load my PRs");
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.my_prs.error").to_string(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let text = if prs.is_empty() {
            t!("telegram_bot.commands.my_prs.empty").to_string()
        } else {
            let now = Utc::now();
            let mut builder = MessageBuilder::new()
                .bold(
                    &t!(
                        "telegram_bot.commands.my_prs.title",
                        count = prs.len()
                    )
                    .to_string(),
                )
                .empty_line();

            let mut sorted = prs.clone();
            sorted.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));

            for pr in &sorted {
                let age = now.signed_duration_since(pr.updated_at);
                let age_label = format_age(age);
                builder = builder.with_html_escape(false).raw(&format!(
                    "• #{} <a href=\"{}\">{}</a> ({}) — {}\n",
                    pr.number,
                    MessageBuilder::escape_html(&pr.url),
                    MessageBuilder::escape_html(&pr.title),
                    MessageBuilder::escape_html(&pr.repo),
                    age_label,
                ));
            }
            builder.build()
        };

        self.context
            .bot
            .send_message(self.context.msg.chat.id, text)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
}

fn format_age(d: chrono::Duration) -> String {
    let hours = d.num_hours();
    if hours < 1 {
        let minutes = d.num_minutes().max(1);
        format!("{} мин", minutes)
    } else if hours < 24 {
        format!("{} ч", hours)
    } else {
        format!("{} дн", hours / 24)
    }
}
