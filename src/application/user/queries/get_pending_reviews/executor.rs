use crate::application::user::queries::get_pending_reviews::error::GetPendingReviewsError;
use crate::application::user::queries::get_pending_reviews::query::GetPendingReviewsQuery;
use crate::application::user::queries::get_pending_reviews::response::GetPendingReviewsResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_connection_repositories_repository::UserConnectionRepositoriesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use std::sync::Arc;

pub struct GetPendingReviewsExecutor {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
    repository_repo: Arc<dyn RepositoryRepository>,
    version_control_client: Arc<dyn VersionControlClient>,
    reversible_cipher: Arc<ReversibleCipher>,
}

impl GetPendingReviewsExecutor {
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
        user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
        repository_repo: Arc<dyn RepositoryRepository>,
        version_control_client: Arc<dyn VersionControlClient>,
        reversible_cipher: Arc<ReversibleCipher>,
    ) -> Self {
        Self {
            user_socials_repo,
            user_vc_accounts_repo,
            user_connection_repositories_repo,
            repository_repo,
            version_control_client,
            reversible_cipher,
        }
    }
}

impl CommandExecutor for GetPendingReviewsExecutor {
    type Command = GetPendingReviewsQuery;
    type Response = GetPendingReviewsResponse;
    type Error = GetPendingReviewsError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await
            .map_err(|_| GetPendingReviewsError::UserNotFound)?;

        let vc = self
            .user_vc_accounts_repo
            .find_by_user_id(&social.user_id)
            .await
            .map_err(|_| GetPendingReviewsError::NoGithubAccount)?;

        let token = self
            .reversible_cipher
            .decrypt(vc.access_token.value())
            .map_err(|e| GetPendingReviewsError::GithubError(e.to_string()))?;

        let connections = self
            .user_connection_repositories_repo
            .find_by_user_id(social.user_id)
            .await
            .map_err(|e| GetPendingReviewsError::DbError(e.to_string()))?;

        let mut repos: Vec<String> = Vec::new();
        for conn in connections {
            if let Ok(repo) = self.repository_repo.find_by_id(conn.repository_id).await {
                repos.push(format!("{}/{}", repo.owner, repo.name));
            }
        }

        let prs = self
            .version_control_client
            .search_user_pending_reviews(&token, &vc.version_control_login, &repos)
            .await
            .map_err(|e| GetPendingReviewsError::GithubError(e.to_string()))?;

        Ok(GetPendingReviewsResponse { prs })
    }
}
