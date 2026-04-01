use crate::application::digest::queries::get_user_digest_subscriptions::error::GetUserDigestSubscriptionsError;
use crate::application::digest::queries::get_user_digest_subscriptions::query::GetUserDigestSubscriptionsQuery;
use crate::application::digest::queries::get_user_digest_subscriptions::response::GetUserDigestSubscriptionsResponse;
use crate::domain::digest::repositories::digest_subscription_repository::DigestSubscriptionRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use std::sync::Arc;

pub struct GetUserDigestSubscriptionsExecutor {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
}

impl GetUserDigestSubscriptionsExecutor {
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

impl CommandExecutor for GetUserDigestSubscriptionsExecutor {
    type Command = GetUserDigestSubscriptionsQuery;
    type Response = GetUserDigestSubscriptionsResponse;
    type Error = GetUserDigestSubscriptionsError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_account = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;

        let subscriptions = self
            .digest_subscription_repo
            .find_by_user_id(social_account.user_id)
            .await?;

        Ok(GetUserDigestSubscriptionsResponse { subscriptions })
    }
}
