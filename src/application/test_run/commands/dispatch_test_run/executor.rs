use crate::application::test_run::commands::dispatch_test_run::command::DispatchTestRunCommand;
use crate::application::test_run::commands::dispatch_test_run::error::DispatchTestRunError;
use crate::application::test_run::commands::dispatch_test_run::response::DispatchTestRunResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_run::NewTestRun;
use crate::domain::test_run::ports::test_runner::TestRunner;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use crate::domain::test_run::value_objects::run_tag::RunTag;
use std::sync::Arc;

pub struct DispatchTestRunExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
    test_run_repo: Arc<dyn TestRunRepository>,
    test_runner: Arc<dyn TestRunner>,
}

impl DispatchTestRunExecutor {
    pub fn new(
        test_suite_repo: Arc<dyn TestSuiteRepository>,
        test_run_repo: Arc<dyn TestRunRepository>,
        test_runner: Arc<dyn TestRunner>,
    ) -> Self {
        Self {
            test_suite_repo,
            test_run_repo,
            test_runner,
        }
    }
}

impl CommandExecutor for DispatchTestRunExecutor {
    type Command = DispatchTestRunCommand;
    type Response = DispatchTestRunResponse;
    type Error = DispatchTestRunError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let suite = self
            .test_suite_repo
            .find_by_repository(cmd.repository_id)
            .await?;

        // Стенд один: пока прогон идёт, второй запуск только занял бы очередь
        if let Some(active) = self.test_run_repo.find_active(cmd.repository_id).await? {
            return Err(DispatchTestRunError::AlreadyRunning(Box::new(active)));
        }

        let git_ref = cmd
            .git_ref
            .clone()
            .unwrap_or_else(|| suite.default_ref.clone());
        let run_tag = RunTag::generate(chrono::Utc::now());

        // Запись создаётся до отправки: иначе пришедший вебхук не с чем связать
        let run = self
            .test_run_repo
            .create(&NewTestRun {
                repository_id: cmd.repository_id,
                run_tag: run_tag.clone(),
                git_ref: git_ref.clone(),
                trigger: cmd.trigger,
                args: match cmd.args.is_empty() {
                    true => None,
                    false => Some(cmd.args.clone()),
                },
                requested_by_user_id: cmd.requested_by_user_id,
                chat_id: cmd.chat_id,
            })
            .await?;

        self.test_runner
            .dispatch(&suite, &git_ref, &cmd.args, &run_tag)
            .await?;

        tracing::info!(
            repository_id = %cmd.repository_id.0,
            run_tag = %run_tag,
            git_ref = %git_ref,
            args = %cmd.args,
            "Test run dispatched"
        );

        Ok(DispatchTestRunResponse { run })
    }
}
