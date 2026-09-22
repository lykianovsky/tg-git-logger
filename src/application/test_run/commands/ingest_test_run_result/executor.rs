use crate::application::test_run::commands::ingest_test_run_result::command::IngestTestRunResultCommand;
use crate::application::test_run::commands::ingest_test_run_result::error::IngestTestRunResultError;
use crate::application::test_run::commands::ingest_test_run_result::response::IngestTestRunResultResponse;
use crate::application::test_run::service::ci_token::CiTokenResolver;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_failure::NewTestFailure;
use crate::domain::test_run::entities::test_run::TestRunOutcome;
use crate::domain::test_run::ports::test_runner::{TestRunArtifacts, TestRunner};
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use crate::domain::test_run::value_objects::test_fingerprint::TestFingerprint;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use std::sync::Arc;

pub struct IngestTestRunResultExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
    test_run_repo: Arc<dyn TestRunRepository>,
    test_runner: Arc<dyn TestRunner>,
    ci_token_resolver: Arc<CiTokenResolver>,
}

impl IngestTestRunResultExecutor {
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

    fn build_failures(
        repository_id: RepositoryId,
        artifacts: &TestRunArtifacts,
    ) -> Vec<NewTestFailure> {
        artifacts
            .failures
            .iter()
            .map(|failure| NewTestFailure {
                project: failure.project.clone(),
                file: failure.file.clone(),
                title: failure.title.clone(),
                fingerprint: TestFingerprint::build(
                    repository_id,
                    &failure.project,
                    &failure.file,
                    &failure.title,
                ),
                error_excerpt: failure.error_excerpt.clone(),
            })
            .collect()
    }

    /// Итоги отчёта точнее статуса CI: шаг сборки мог упасть уже после тестов
    fn resolve_status(outcome: &TestRunOutcome) -> TestRunStatus {
        if outcome.status == TestRunStatus::Cancelled {
            return TestRunStatus::Cancelled;
        }

        match outcome.totals {
            Some(totals) if totals.failed > 0 => TestRunStatus::Failed,
            Some(_) => TestRunStatus::Passed,
            None => TestRunStatus::Unknown,
        }
    }
}

impl CommandExecutor for IngestTestRunResultExecutor {
    type Command = IngestTestRunResultCommand;
    type Response = IngestTestRunResultResponse;
    type Error = IngestTestRunResultError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let run = self.test_run_repo.find_by_tag(&cmd.run_tag).await?;
        let suite = self
            .test_suite_repo
            .find_by_repository(run.repository_id)
            .await?;

        // Итоги читаем правами того, кто запустил прогон; ночной — правами администратора
        let actor = self
            .ci_token_resolver
            .resolve_or_background(run.requested_by_user_id)
            .await?;

        let mut outcome = self
            .test_runner
            .find_run_by_tag(&actor.token, &suite, &cmd.run_tag)
            .await?;

        // Прогон ещё идёт: запоминаем ссылку на него, итоги придут следующим вебхуком
        if outcome.status.is_active() {
            if let Some(provider_run_id) = outcome.provider_run_id {
                self.test_run_repo
                    .mark_started(
                        run.id,
                        provider_run_id,
                        outcome.run_url.clone().unwrap_or_default(),
                    )
                    .await?;
            }

            return Ok(IngestTestRunResultResponse {
                run: self.test_run_repo.find_by_id(run.id).await?,
                summary_missing: false,
            });
        }

        let mut failures = Vec::new();

        if let Some(provider_run_id) = outcome.provider_run_id {
            let artifacts = self
                .test_runner
                .fetch_artifacts(&actor.token, &suite, provider_run_id)
                .await?;

            outcome.totals = artifacts.totals;

            failures = Self::build_failures(run.repository_id, &artifacts);
        }

        outcome.status = Self::resolve_status(&outcome);

        self.test_run_repo.save_outcome(run.id, &outcome).await?;
        // Повторный вебхук того же прогона не должен задваивать список упавших
        self.test_run_repo
            .replace_failures(run.id, &failures)
            .await?;

        let updated = self.test_run_repo.find_by_id(run.id).await?;

        tracing::info!(
            run_tag = %cmd.run_tag.as_str(),
            status = %updated.status.as_str(),
            failures = failures.len(),
            "Test run result ingested"
        );

        Ok(IngestTestRunResultResponse {
            run: updated,
            summary_missing: outcome.totals.is_none(),
        })
    }
}
