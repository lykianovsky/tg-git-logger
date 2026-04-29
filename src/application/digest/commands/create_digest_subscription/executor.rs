use crate::application::digest::commands::create_digest_subscription::command::CreateDigestSubscriptionCommand;
use crate::application::digest::commands::create_digest_subscription::error::CreateDigestSubscriptionExecutorError;
use crate::application::digest::commands::create_digest_subscription::response::CreateDigestSubscriptionResponse;
use crate::domain::digest::entities::digest_subscription::DigestSubscription;
use crate::domain::digest::repositories::digest_subscription_repository::DigestSubscriptionRepository;
use crate::domain::digest::value_objects::digest_subscription_id::DigestSubscriptionId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use chrono::Utc;
use std::sync::Arc;

pub struct CreateDigestSubscriptionExecutor {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
}

impl CreateDigestSubscriptionExecutor {
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
    ) -> Self {
        Self {
            user_socials_repo,
            digest_subscription_repo,
        }
    }
}

impl CommandExecutor for CreateDigestSubscriptionExecutor {
    type Command = CreateDigestSubscriptionCommand;
    type Response = CreateDigestSubscriptionResponse;
    type Error = CreateDigestSubscriptionExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_account = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;

        let subscription = DigestSubscription {
            id: DigestSubscriptionId::default(),
            user_id: social_account.user_id,
            repository_id: cmd.repository_id,
            digest_type: cmd.digest_type.clone(),
            send_hour: cmd.send_hour,
            send_minute: cmd.send_minute,
            day_of_week: cmd.day_of_week,
            is_active: true,
            last_sent_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = self.digest_subscription_repo.create(&subscription).await?;

        Ok(CreateDigestSubscriptionResponse {
            subscription: created,
        })
    }
}
