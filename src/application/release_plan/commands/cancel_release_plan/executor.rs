use crate::application::release_plan::commands::cancel_release_plan::command::CancelReleasePlanExecutorCommand;
use crate::application::release_plan::commands::cancel_release_plan::error::CancelReleasePlanExecutorError;
use crate::application::release_plan::commands::cancel_release_plan::response::CancelReleasePlanExecutorResponse;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::release_plan::value_objects::release_plan_status::ReleasePlanStatus;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;

pub struct CancelReleasePlanExecutor {
    pub release_plan_repo: Arc<dyn ReleasePlanRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

impl CommandExecutor for CancelReleasePlanExecutor {
    type Command = CancelReleasePlanExecutorCommand;
    type Response = CancelReleasePlanExecutorResponse;
    type Error = CancelReleasePlanExecutorError;

    #[tracing::instrument(
        name = "cancel_release_plan",
        skip_all,
        fields(plan_id = cmd.plan_id.0)
    )]
    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let plan = self.release_plan_repo.find_by_id(cmd.plan_id).await?;

        self.release_plan_repo
            .set_status(cmd.plan_id, ReleasePlanStatus::Cancelled)
            .await?;

        if let Some(chat_id) = plan.announce_chat_id {
            let canceller = self
                .user_socials_repo
                .find_by_social_user_id(&cmd.cancelled_by_social_user_id)
                .await?;
            let canceller_label = canceller
                .social_user_login
                .map(|n| format!("@{}", n))
                .unwrap_or_else(|| canceller.social_user_id.0.to_string());

            let msg = MessageBuilder::new()
                .bold(
                    &t!(
                        "telegram_bot.notifications.release_plan.cancelled.title",
                        date = plan.planned_date.format("%d.%m.%Y").to_string()
                    )
                    .to_string(),
                )
                .empty_line()
                .with_html_escape(true)
                .section(
                    &t!("telegram_bot.notifications.release_plan.cancelled.by").to_string(),
                    &canceller_label,
                )
                .section(
                    &t!("telegram_bot.notifications.release_plan.cancelled.reason").to_string(),
                    &cmd.reason,
                );

            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id,
                    message: msg,
                })
                .await
                .ok();
        }

        Ok(CancelReleasePlanExecutorResponse)
    }
}
