use crate::application::digest::commands::update_digest_subscription::command::UpdateDigestSubscriptionCommand;
use crate::application::digest::commands::update_digest_subscription::error::UpdateDigestSubscriptionExecutorError;
use crate::application::digest::commands::update_digest_subscription::response::UpdateDigestSubscriptionResponse;
use crate::domain::digest::repositories::digest_subscription_repository::DigestSubscriptionRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct UpdateDigestSubscriptionExecutor {
    digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
}

impl UpdateDigestSubscriptionExecutor {
    pub fn new(digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>) -> Self {
        Self {
            digest_subscription_repo,
        }
    }
}

impl CommandExecutor for UpdateDigestSubscriptionExecutor {
    type Command = UpdateDigestSubscriptionCommand;
    type Response = UpdateDigestSubscriptionResponse;
    type Error = UpdateDigestSubscriptionExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut subscription = self.digest_subscription_repo.find_by_id(cmd.id).await?;

        if let Some(is_active) = cmd.is_active {
            subscription.is_active = is_active;
        }

        if let Some(hour) = cmd.send_hour {
            subscription.send_hour = hour;
        }

        if let Some(minute) = cmd.send_minute {
            subscription.send_minute = minute;
        }

        self.digest_subscription_repo.update(&subscription).await?;

        Ok(UpdateDigestSubscriptionResponse)
    }
}
