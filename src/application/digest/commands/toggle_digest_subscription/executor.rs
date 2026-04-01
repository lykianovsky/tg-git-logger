use crate::application::digest::commands::toggle_digest_subscription::command::ToggleDigestSubscriptionCommand;
use crate::application::digest::commands::toggle_digest_subscription::error::ToggleDigestSubscriptionExecutorError;
use crate::application::digest::commands::toggle_digest_subscription::response::ToggleDigestSubscriptionResponse;
use crate::domain::digest::repositories::digest_subscription_repository::DigestSubscriptionRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct ToggleDigestSubscriptionExecutor {
    digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
}

impl ToggleDigestSubscriptionExecutor {
    pub fn new(digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>) -> Self {
        Self {
            digest_subscription_repo,
        }
    }
}

impl CommandExecutor for ToggleDigestSubscriptionExecutor {
    type Command = ToggleDigestSubscriptionCommand;
    type Response = ToggleDigestSubscriptionResponse;
    type Error = ToggleDigestSubscriptionExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut subscription = self
            .digest_subscription_repo
            .find_by_id(cmd.id)
            .await?;

        subscription.is_active = !subscription.is_active;

        self.digest_subscription_repo
            .update(&subscription)
            .await?;

        Ok(ToggleDigestSubscriptionResponse {
            is_active: subscription.is_active,
        })
    }
}
