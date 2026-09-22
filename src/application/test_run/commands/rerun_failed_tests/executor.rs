use crate::application::test_run::commands::dispatch_test_run::command::DispatchTestRunCommand;
use crate::application::test_run::commands::dispatch_test_run::executor::DispatchTestRunExecutor;
use crate::application::test_run::commands::rerun_failed_tests::command::RerunFailedTestsCommand;
use crate::application::test_run::commands::rerun_failed_tests::error::RerunFailedTestsError;
use crate::application::test_run::commands::rerun_failed_tests::response::RerunFailedTestsResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_failure::TestFailure;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::domain::test_run::value_objects::test_run_trigger::TestRunTrigger;
use std::sync::Arc;

/// Вход workflow не резиновый: если фильтр по названиям не влезает,
/// гоняем файлы целиком — это всё равно много меньше полного набора
const MAX_ARGS_LENGTH: usize = 900;

pub struct RerunFailedTestsExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
    dispatch_test_run: Arc<DispatchTestRunExecutor>,
}

impl RerunFailedTestsExecutor {
    pub fn new(
        test_run_repo: Arc<dyn TestRunRepository>,
        dispatch_test_run: Arc<DispatchTestRunExecutor>,
    ) -> Self {
        Self {
            test_run_repo,
            dispatch_test_run,
        }
    }

    /// Названия тестов уходят в `--grep`, поэтому спецсимволы регулярки экранируем
    fn escape_pattern(title: &str) -> String {
        title
            .chars()
            .flat_map(|symbol| match symbol {
                '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^'
                | '$' | '/' => vec!['\\', symbol],
                _ => vec![symbol],
            })
            .collect()
    }

    fn build_args(failures: &[TestFailure]) -> String {
        let pattern = failures
            .iter()
            .map(|failure| Self::escape_pattern(&failure.title))
            .collect::<Vec<_>>()
            .join("|");
        let args = format!("--grep \"{pattern}\"");

        if args.len() <= MAX_ARGS_LENGTH {
            return args;
        }

        let mut files: Vec<String> = failures
            .iter()
            .map(|failure| failure.file.clone())
            .collect();

        files.sort();
        files.dedup();

        files.join(" ")
    }
}

impl CommandExecutor for RerunFailedTestsExecutor {
    type Command = RerunFailedTestsCommand;
    type Response = RerunFailedTestsResponse;
    type Error = RerunFailedTestsError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let run = self
            .test_run_repo
            .find_last(cmd.repository_id)
            .await?
            .ok_or(RerunFailedTestsError::NothingToRerun)?;
        let failures = self.test_run_repo.list_failures(run.id).await?;

        if failures.is_empty() {
            return Err(RerunFailedTestsError::NothingToRerun);
        }

        let dispatched = self
            .dispatch_test_run
            .execute(&DispatchTestRunCommand {
                repository_id: cmd.repository_id,
                git_ref: Some(run.git_ref.clone()),
                args: Self::build_args(&failures),
                trigger: TestRunTrigger::Chat,
                requested_by_social_user_id: Some(cmd.social_user_id),
                chat_id: Some(cmd.chat_id),
            })
            .await?;

        Ok(RerunFailedTestsResponse {
            run: dispatched.run,
            failures_count: failures.len(),
        })
    }
}
