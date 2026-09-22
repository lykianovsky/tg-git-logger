use crate::application::test_run::commands::cancel_test_run::command::CancelTestRunCommand;
use crate::application::test_run::commands::cancel_test_run::error::CancelTestRunError;
use crate::application::test_run::commands::cancel_test_run::response::CancelTestRunResponse;
use crate::application::test_run::service::ci_token::CiTokenResolver;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_run::TestRunOutcome;
use crate::domain::test_run::ports::test_runner::TestRunner;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use std::sync::Arc;

pub struct CancelTestRunExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
    test_run_repo: Arc<dyn TestRunRepository>,
    test_runner: Arc<dyn TestRunner>,
    ci_token_resolver: Arc<CiTokenResolver>,
}

impl CancelTestRunExecutor {
    pub fn new(
        test_suite_repo: Arc<dyn TestSuiteRepository>,
        test_run_repo: Arc<dyn TestRunRepository>,
        test_runner: Arc<dyn TestRunner>,
        ci_token_resolver: Arc<CiTokenResolver>,
    ) -> Self {
        Self {
            test_suite_repo,
            test_run_repo,
            test_runner,
            ci_token_resolver,
        }
    }
}

impl CommandExecutor for CancelTestRunExecutor {
    type Command = CancelTestRunCommand;
    type Response = CancelTestRunResponse;
    type Error = CancelTestRunError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let run = self
            .test_run_repo
            .find_active(cmd.repository_id)
            .await?
            .ok_or(CancelTestRunError::NoActiveRun)?;

        // Прогон ещё не появился в CI — отменять нечего, но и висеть он не должен
        let Some(provider_run_id) = run.provider_run_id else {
            return Err(CancelTestRunError::NoActiveRun);
        };

        let suite = self
            .test_suite_repo
            .find_by_repository(cmd.repository_id)
            .await?;
        let actor = self
            .ci_token_resolver
            .resolve_by_social_user_id(&cmd.social_user_id)
            .await?;

        self.test_runner
            .cancel_run(&actor.token, &suite, provider_run_id)
            .await?;

        self.test_run_repo
            .save_outcome(
                run.id,
                &TestRunOutcome {
                    status: TestRunStatus::Cancelled,
                    provider_run_id: Some(provider_run_id),
                    run_url: run.run_url.clone(),
                    sha: run.sha.clone(),
                    started_at: run.started_at,
                    finished_at: Some(chrono::Utc::now()),
                    totals: run.totals,
                },
            )
            .await?;

        tracing::info!(run_tag = %run.run_tag, "Test run cancelled");

        Ok(CancelTestRunResponse {
            run: self.test_run_repo.find_by_id(run.id).await?,
        })
    }
}
