use crate::application::release_plan::commands::send_release_day_reminders::command::SendReleaseDayRemindersExecutorCommand;
use crate::application::release_plan::commands::send_release_day_reminders::error::SendReleaseDayRemindersExecutorError;
use crate::application::release_plan::commands::send_release_day_reminders::response::SendReleaseDayRemindersExecutorResponse;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::release_plan::entities::release_plan::ReleasePlanNotificationKind;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use chrono::Utc;
use chrono_tz::Europe::Moscow;
use std::sync::Arc;

pub struct SendReleaseDayRemindersExecutor {
    pub release_plan_repo: Arc<dyn ReleasePlanRepository>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

impl CommandExecutor for SendReleaseDayRemindersExecutor {
    type Command = SendReleaseDayRemindersExecutorCommand;
    type Response = SendReleaseDayRemindersExecutorResponse;
    type Error = SendReleaseDayRemindersExecutorError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let today_msk = Utc::now().with_timezone(&Moscow).date_naive();

        let plans = self
            .release_plan_repo
            .find_due_for_release_day_reminder(today_msk)
            .await
            .map_err(|e| SendReleaseDayRemindersExecutorError::DbError(e.to_string()))?;

        if plans.is_empty() {
            return Ok(SendReleaseDayRemindersExecutorResponse { sent_count: 0 });
        }

        let mut sent = 0usize;

        for plan in &plans {
            let chat_id = match plan.announce_chat_id {
                Some(c) => c,
                None => {
                    tracing::warn!(
                        plan_id = plan.id.0,
                        "Release day reminder skipped: no announce_chat_id"
                    );
                    continue;
                }
            };

            let mut repo_names: Vec<String> = Vec::with_capacity(plan.repository_ids.len());
            for rid in &plan.repository_ids {
                if let Ok(repo) = self.repository_repo.find_by_id(*rid).await {
                    repo_names.push(format!("{}/{}", repo.owner, repo.name));
                }
            }

            let mut msg = MessageBuilder::new()
                .bold(
                    &t!("telegram_bot.notifications.release_plan.today_reminder.title").to_string(),
                )
                .empty_line()
                .with_html_escape(true)
                .section(
                    &t!("telegram_bot.notifications.release_plan.repos").to_string(),
                    &if repo_names.is_empty() {
                        "—".to_string()
                    } else {
                        repo_names.join(", ")
                    },
                );

            if let Some(note) = &plan.note {
                msg = msg.section(
                    &t!("telegram_bot.notifications.release_plan.note").to_string(),
                    note,
                );
            }

            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id,
                    message: msg,
                })
                .await
                .ok();

            if let Err(e) = self
                .release_plan_repo
                .mark_notified(plan.id, ReleasePlanNotificationKind::ReleaseDay, Utc::now())
                .await
            {
                tracing::error!(
                    error = %e,
                    plan_id = plan.id.0,
                    "Failed to mark release plan as notified (release day)"
                );
                continue;
            }

            sent += 1;
        }

        Ok(SendReleaseDayRemindersExecutorResponse { sent_count: sent })
    }
}
