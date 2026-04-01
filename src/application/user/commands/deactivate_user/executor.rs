use crate::application::user::commands::deactivate_user::command::DeactivateUserCommand;
use crate::application::user::commands::deactivate_user::error::DeactivateUserExecutorError;
use crate::application::user::commands::deactivate_user::response::DeactivateUserResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_repository::UserRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use std::sync::Arc;

pub struct DeactivateUserExecutor {
    user_repo: Arc<dyn UserRepository>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
}

impl DeactivateUserExecutor {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    ) -> Self {
        Self {
            user_repo,
            user_socials_repo,
        }
    }
}

impl CommandExecutor for DeactivateUserExecutor {
    type Command = DeactivateUserCommand;
    type Response = DeactivateUserResponse;
    type Error = DeactivateUserExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_account = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;

        self.user_repo
            .set_active(social_account.user_id, false)
            .await?;

        Ok(DeactivateUserResponse)
    }
}
