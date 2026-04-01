use crate::application::digest::commands::delete_digest_subscription::command::DeleteDigestSubscriptionCommand;
use crate::application::digest::commands::delete_digest_subscription::error::DeleteDigestSubscriptionExecutorError;
use crate::application::digest::commands::delete_digest_subscription::response::DeleteDigestSubscriptionResponse;
use crate::domain::digest::repositories::digest_subscription_repository::DigestSubscriptionRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct DeleteDigestSubscriptionExecutor {
    digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
}

impl DeleteDigestSubscriptionExecutor {
    pub fn new(digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>) -> Self {
        Self {
            digest_subscription_repo,
        }
    }
}

impl CommandExecutor for DeleteDigestSubscriptionExecutor {
    type Command = DeleteDigestSubscriptionCommand;
    type Response = DeleteDigestSubscriptionResponse;
    type Error = DeleteDigestSubscriptionExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.digest_subscription_repo.delete(cmd.id).await?;

        Ok(DeleteDigestSubscriptionResponse)
    }
}
