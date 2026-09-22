use crate::application::test_run::queries::list_ci_options::error::ListCiOptionsError;
use crate::application::test_run::queries::list_ci_options::query::{
    CiOptionKind, ListCiOptionsQuery,
};
use crate::application::test_run::queries::list_ci_options::response::ListCiOptionsResponse;
use crate::application::test_run::service::ci_token::CiTokenResolver;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::ports::test_runner::TestRunner;
use std::sync::Arc;

pub struct ListCiOptionsExecutor {
    repository_repo: Arc<dyn RepositoryRepository>,
    test_runner: Arc<dyn TestRunner>,
    ci_token_resolver: Arc<CiTokenResolver>,
}

impl ListCiOptionsExecutor {
    pub fn new(
        repository_repo: Arc<dyn RepositoryRepository>,
        test_runner: Arc<dyn TestRunner>,
        ci_token_resolver: Arc<CiTokenResolver>,
    ) -> Self {
        Self {
            repository_repo,
            test_runner,
            ci_token_resolver,
        }
    }
}

impl CommandExecutor for ListCiOptionsExecutor {
    type Command = ListCiOptionsQuery;
    type Response = ListCiOptionsResponse;
    type Error = ListCiOptionsError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let repository = self.repository_repo.find_by_id(query.repository_id).await?;
        let actor = self
            .ci_token_resolver
            .resolve_by_social_user_id(&query.social_user_id)
            .await?;

        let options = match query.kind {
            CiOptionKind::Workflows => {
                self.test_runner
                    .list_workflows(&actor.token, &repository.owner, &repository.name)
                    .await?
            }
        };

        Ok(ListCiOptionsResponse { options })
    }
}
