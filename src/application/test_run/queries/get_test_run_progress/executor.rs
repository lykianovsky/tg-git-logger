use crate::application::test_run::queries::get_test_run_progress::error::GetTestRunProgressError;
use crate::application::test_run::queries::get_test_run_progress::query::GetTestRunProgressQuery;
use crate::application::test_run::queries::get_test_run_progress::response::GetTestRunProgressResponse;
use crate::application::test_run::service::ci_token::CiTokenResolver;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::ports::test_runner::TestRunner;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use std::sync::Arc;

pub struct GetTestRunProgressExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
    test_runner: Arc<dyn TestRunner>,
    ci_token_resolver: Arc<CiTokenResolver>,
}

impl GetTestRunProgressExecutor {
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

impl CommandExecutor for GetTestRunProgressExecutor {
    type Command = GetTestRunProgressQuery;
    type Response = GetTestRunProgressResponse;
    type Error = GetTestRunProgressError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        // Прогон ещё не появился в CI — показывать нечего
        let Some(provider_run_id) = query.run.provider_run_id else {
            return Ok(GetTestRunProgressResponse { progress: None });
        };

        let suite = self
            .test_suite_repo
            .find_by_repository(query.run.repository_id)
            .await?;
        let actor = self
            .ci_token_resolver
            .resolve_or_background(query.run.requested_by_user_id)
            .await?;

        let progress = self
            .test_runner
            .fetch_progress(&actor.token, &suite, provider_run_id)
            .await?;

        Ok(GetTestRunProgressResponse { progress })
    }
}
