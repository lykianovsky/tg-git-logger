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
use crate::delivery::bot::telegram::keyboards::actions::release_plan_settings::{
    rps_cancel_btn_callback, rps_complete_btn_callback, rps_select_callback,
};
use crate::domain::release_plan::entities::release_plan::ReleasePlan;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::role::value_objects::role_name::RoleName;
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

        let roles = match self
            .executors
            .queries
            .get_user_roles_by_telegram_id
            .execute(&GetUserRolesByTelegramIdQuery { social_user_id })
            .await
        {
            Ok(r) => r.roles,
            Err(e) => {
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
        };

        let can_manage = roles.contains(&RoleName::Admin)
            || roles.contains(&RoleName::ProductManager);

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

        self.context
            .bot
            .send_message(self.context.msg.chat.id, header)
            .parse_mode(ParseMode::Html)
            .await?;

        for plan in &plans {
            let text = render_plan_card(plan, &repo_label_by_id, today_msk);
            let mut send = self
                .context
                .bot
                .send_message(self.context.msg.chat.id, text)
                .parse_mode(ParseMode::Html);

            if can_manage {
                send = send.reply_markup(build_plan_actions_keyboard(plan.id.0));
            }
            send.await?;
        }

        if can_manage {
            self.dialogue
                .update(TelegramBotDialogueState::ReleasePlanSettings(
                    TelegramBotReleasePlanSettingsState::AwaitingSelection,
                ))
                .await?;
        }

        Ok(())
    }
}

fn build_plan_actions_keyboard(plan_id: i32) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback(
            t!("telegram_bot.commands.releases.btn_edit").to_string(),
            rps_select_callback(plan_id),
        ),
        InlineKeyboardButton::callback(
            t!("telegram_bot.commands.releases.btn_cancel").to_string(),
            rps_cancel_btn_callback(plan_id),
        ),
        InlineKeyboardButton::callback(
            t!("telegram_bot.commands.releases.btn_complete").to_string(),
            rps_complete_btn_callback(plan_id),
        ),
    ]])
}

fn render_plan_card(
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

fn format_date_label(planned: NaiveDate, today: NaiveDate) -> String {
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
