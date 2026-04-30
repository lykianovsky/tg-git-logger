use crate::application::release_plan::queries::get_upcoming_release_plans::query::GetUpcomingReleasePlansQuery;
use crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery;
use crate::application::user::queries::get_user_roles_by_telegram_id::error::GetUserRolesByTelegramIdError;
use crate::application::user::queries::get_user_roles_by_telegram_id::query::GetUserRolesByTelegramIdQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::release_plan_settings::TelegramBotReleasePlanSettingsState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::release_plan_settings::rps_view_callback;
use crate::domain::release_plan::entities::release_plan::ReleasePlan;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::builder::message::MessageBuilder;
use chrono::{NaiveDate, Utc};
use chrono_tz::Europe::Moscow;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub struct TelegramBotReleasesCommandHandler {
    context: TelegramBotCommandContext,
    executors: Arc<ApplicationBoostrapExecutors>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotReleasesCommandHandler {
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

        if let Err(e) = self
            .executors
            .queries
            .get_user_roles_by_telegram_id
            .execute(&GetUserRolesByTelegramIdQuery { social_user_id })
            .await
        {
            let reply = match e {
                GetUserRolesByTelegramIdError::UserNotFound => {
                    t!("telegram_bot.commands.releases.not_registered").to_string()
                }
                GetUserRolesByTelegramIdError::DbError(_) => {
                    tracing::error!(error = %e, "Failed to load user roles");
                    t!("telegram_bot.commands.releases.error").to_string()
                }
            };
            self.context
                .bot
                .send_message(self.context.msg.chat.id, reply)
                .await?;
            return Ok(());
        }

        let today_msk = Utc::now().with_timezone(&Moscow).date_naive();

        let plans = match self
            .executors
            .queries
            .get_upcoming_release_plans
            .execute(&GetUpcomingReleasePlansQuery {
                from_date: today_msk,
            })
            .await
        {
            Ok(r) => r.plans,
            Err(e) => {
                tracing::error!(error = %e, "Failed to load upcoming release plans");
                self.context
                    .bot
                    .send_message(
                        self.context.msg.chat.id,
                        t!("telegram_bot.commands.releases.error").to_string(),
                    )
                    .await?;
                return Ok(());
            }
        };

        if plans.is_empty() {
            self.context
                .bot
                .send_message(
                    self.context.msg.chat.id,
                    t!("telegram_bot.commands.releases.empty").to_string(),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            return Ok(());
        }

        let repos = self
            .executors
            .queries
            .get_all_repositories
            .execute(&GetAllRepositoriesQuery {})
            .await
            .map(|r| r.repositories)
            .unwrap_or_default();

        let repo_label_by_id: HashMap<RepositoryId, String> = repos
            .into_iter()
            .map(|r| (r.id, format!("{}/{}", r.owner, r.name)))
            .collect();

        let header = t!(
            "telegram_bot.commands.releases.title",
            count = plans.len()
        )
        .to_string();

        let keyboard = build_plans_list_keyboard(&plans, &repo_label_by_id, today_msk);

        self.context
            .bot
            .send_message(self.context.msg.chat.id, header)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;

        self.dialogue
            .update(TelegramBotDialogueState::ReleasePlanSettings(
                TelegramBotReleasePlanSettingsState::AwaitingSelection,
            ))
            .await?;

        Ok(())
    }
}

pub fn build_plans_list_keyboard(
    plans: &[ReleasePlan],
    repo_label_by_id: &HashMap<RepositoryId, String>,
    today_msk: NaiveDate,
) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = plans
        .iter()
        .map(|plan| {
            let label = build_plan_button_label(plan, repo_label_by_id, today_msk);
            vec![InlineKeyboardButton::callback(
                label,
                rps_view_callback(plan.id.0),
            )]
        })
        .collect();
    InlineKeyboardMarkup::new(rows)
}

fn build_plan_button_label(
    plan: &ReleasePlan,
    repo_label_by_id: &HashMap<RepositoryId, String>,
    today_msk: NaiveDate,
) -> String {
    let date_part = plan.planned_date.format("%d.%m").to_string();
    let day_label = format_date_label(plan.planned_date, today_msk);

    let repos_summary = if plan.repository_ids.is_empty() {
        "—".to_string()
    } else if plan.repository_ids.len() <= 2 {
        plan.repository_ids
            .iter()
            .map(|id| {
                repo_label_by_id
                    .get(id)
                    .map(|s| s.split('/').next_back().unwrap_or(s.as_str()).to_string())
                    .unwrap_or_else(|| format!("#{}", id.0))
            })
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        let first = plan
            .repository_ids
            .first()
            .and_then(|id| repo_label_by_id.get(id))
            .map(|s| s.split('/').next_back().unwrap_or(s.as_str()).to_string())
            .unwrap_or_else(|| "?".to_string());
        format!("{} +{}", first, plan.repository_ids.len() - 1)
    };

    format!("📅 {} ({}) — {}", date_part, day_label, repos_summary)
}

pub fn render_plan_card(
    plan: &ReleasePlan,
    repo_label_by_id: &HashMap<RepositoryId, String>,
    today_msk: NaiveDate,
) -> String {
    let date_label = format_date_label(plan.planned_date, today_msk);

    let mut builder = MessageBuilder::new()
        .with_html_escape(false)
        .raw(&format!(
            "🚀 <b>{}</b> — {}\n",
            plan.planned_date.format("%d.%m.%Y"),
            MessageBuilder::escape_html(&date_label),
        ));

    let repos_text = if plan.repository_ids.is_empty() {
        "—".to_string()
    } else {
        plan.repository_ids
            .iter()
            .map(|id| {
                repo_label_by_id
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| format!("#{}", id.0))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    builder = builder.with_html_escape(true).section(
        &t!("telegram_bot.commands.releases.repos").to_string(),
        &repos_text,
    );

    if let Some(call) = plan.call_datetime {
        let call_msk = call.with_timezone(&Moscow);
        builder = builder.section(
            &t!("telegram_bot.commands.releases.call").to_string(),
            &call_msk.format("%d.%m %H:%M МСК").to_string(),
        );
    }

    if let Some(url) = &plan.meeting_url {
        builder = builder.with_html_escape(false).raw(&format!(
            "🔗 <a href=\"{}\">{}</a>\n",
            MessageBuilder::escape_html(url),
            t!("telegram_bot.commands.releases.meeting").to_string(),
        ));
    }

    if let Some(note) = &plan.note {
        builder = builder.with_html_escape(true).section(
            &t!("telegram_bot.commands.releases.note").to_string(),
            note,
        );
    }

    builder.build()
}

pub fn format_date_label(planned: NaiveDate, today: NaiveDate) -> String {
    let diff = (planned - today).num_days();
    if diff == 0 {
        t!("telegram_bot.commands.releases.today").to_string()
    } else if diff == 1 {
        t!("telegram_bot.commands.releases.tomorrow").to_string()
    } else if diff > 1 && diff <= 30 {
        t!(
            "telegram_bot.commands.releases.in_days",
            n = diff
        )
        .to_string()
    } else {
        planned.format("%d.%m.%Y").to_string()
    }
}
