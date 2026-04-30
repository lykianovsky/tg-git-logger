use crate::application::release_plan::commands::send_call_reminders::command::SendCallRemindersExecutorCommand;
use crate::application::release_plan::commands::send_call_reminders::error::SendCallRemindersExecutorError;
use crate::application::release_plan::commands::send_call_reminders::response::SendCallRemindersExecutorResponse;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::release_plan::entities::release_plan::ReleasePlanNotificationKind;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use chrono::{Duration, Utc};
use chrono_tz::Europe::Moscow;
use std::sync::Arc;

pub struct SendCallRemindersExecutor {
    pub release_plan_repo: Arc<dyn ReleasePlanRepository>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

impl CommandExecutor for SendCallRemindersExecutor {
    type Command = SendCallRemindersExecutorCommand;
    type Response = SendCallRemindersExecutorResponse;
    type Error = SendCallRemindersExecutorError;

    #[tracing::instrument(name = "send_call_reminders", skip_all)]
    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let now = Utc::now();
        let in_one_hour = now + Duration::hours(1);

        let plans = self
            .release_plan_repo
            .find_due_for_call_reminder(now, in_one_hour)
            .await
            .map_err(|e| SendCallRemindersExecutorError::DbError(e.to_string()))?;

        if plans.is_empty() {
            return Ok(SendCallRemindersExecutorResponse { sent_count: 0 });
        }

        let mut sent = 0usize;

        for plan in &plans {
            let chat_id = match plan.announce_chat_id {
                Some(c) => c,
                None => continue,
            };
            let call_dt = match plan.call_datetime {
                Some(c) => c,
                None => continue,
            };

            let call_msk = call_dt.with_timezone(&Moscow);
            let mut msg = MessageBuilder::new()
                .bold(
                    &t!("telegram_bot.notifications.release_plan.call_reminder.title").to_string(),
                )
                .empty_line()
                .with_html_escape(true)
                .section(
                    &t!("telegram_bot.notifications.release_plan.call").to_string(),
                    &call_msk.format("%H:%M МСК").to_string(),
                );

            if let Some(url) = &plan.meeting_url {
                msg = msg.with_html_escape(false).raw(&format!(
                    "🔗 <a href=\"{}\">{}</a>\n",
                    MessageBuilder::escape_html(url),
                    t!("telegram_bot.notifications.release_plan.meeting").to_string(),
                ));
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
                .mark_notified(plan.id, ReleasePlanNotificationKind::Call, Utc::now())
                .await
            {
                tracing::error!(
                    error = %e,
                    plan_id = plan.id.0,
                    "Failed to mark release plan as notified (call)"
                );
                continue;
            }

            sent += 1;
        }

        Ok(SendCallRemindersExecutorResponse { sent_count: sent })
    }
}
