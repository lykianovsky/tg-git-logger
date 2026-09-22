use crate::application::test_run::queries::list_test_blocks::error::ListTestBlocksError;
use crate::application::test_run::queries::list_test_blocks::query::ListTestBlocksQuery;
use crate::application::test_run::queries::list_test_blocks::response::ListTestBlocksResponse;
use crate::application::test_run::service::ci_token::CiTokenResolver;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::ports::test_runner::TestRunner;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use std::sync::Arc;

pub struct ListTestBlocksExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
    test_runner: Arc<dyn TestRunner>,
    ci_token_resolver: Arc<CiTokenResolver>,
}

impl ListTestBlocksExecutor {
    pub fn new(
        test_suite_repo: Arc<dyn TestSuiteRepository>,
        test_runner: Arc<dyn TestRunner>,
        ci_token_resolver: Arc<CiTokenResolver>,
    ) -> Self {
        Self {
            test_suite_repo,
            test_runner,
            ci_token_resolver,
        }
    }
}

impl CommandExecutor for ListTestBlocksExecutor {
    type Command = ListTestBlocksQuery;
    type Response = ListTestBlocksResponse;
    type Error = ListTestBlocksError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let suite = self
            .test_suite_repo
            .find_by_repository(query.repository_id)
            .await?;
        let actor = self
            .ci_token_resolver
            .resolve_by_social_user_id(&query.social_user_id)
            .await?;
        let blocks = self.test_runner.list_blocks(&actor.token, &suite).await?;

        Ok(ListTestBlocksResponse { blocks })
    }
}
