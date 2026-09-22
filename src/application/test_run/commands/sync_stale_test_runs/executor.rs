use crate::application::test_run::commands::ingest_test_run_result::command::IngestTestRunResultCommand;
use crate::application::test_run::commands::ingest_test_run_result::executor::IngestTestRunResultExecutor;
use crate::application::test_run::commands::sync_stale_test_runs::command::SyncStaleTestRunsCommand;
use crate::application::test_run::commands::sync_stale_test_runs::error::SyncStaleTestRunsError;
use crate::application::test_run::commands::sync_stale_test_runs::response::SyncStaleTestRunsResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use std::sync::Arc;

/// За один заход разбираем немного прогонов: одновременно их столько не бывает,
/// а поход в CI по каждому стоит запроса
const STALE_RUNS_LIMIT: u64 = 10;

pub struct SyncStaleTestRunsExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
    ingest_test_run_result: Arc<IngestTestRunResultExecutor>,
}

impl SyncStaleTestRunsExecutor {
    pub fn new(
        test_run_repo: Arc<dyn TestRunRepository>,
        ingest_test_run_result: Arc<IngestTestRunResultExecutor>,
    ) -> Self {
        Self {
            test_run_repo,
            ingest_test_run_result,
        }
    }
}

impl CommandExecutor for SyncStaleTestRunsExecutor {
    type Command = SyncStaleTestRunsCommand;
    type Response = SyncStaleTestRunsResponse;
    type Error = SyncStaleTestRunsError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let running = self.test_run_repo.find_all_active(STALE_RUNS_LIMIT).await?;
        let mut finished = Vec::new();
        let mut active = Vec::new();

        for run in running {
            let ingested = self
                .ingest_test_run_result
                .execute(&IngestTestRunResultCommand {
                    run_tag: run.run_tag.clone(),
                    // Вебхука не было — состояние добираем запросом в CI
                    known_state: None,
                })
                .await;

            match ingested {
                // Прогон всё ещё идёт — обновим карточку и вернёмся к нему позже
                Ok(response) if response.run.is_active() => active.push(response.run),
                Ok(response) => finished.push(response.run),
                Err(error) => {
                    tracing::warn!(%error, run_tag = %run.run_tag, "Failed to sync stale test run");
                }
            }
        }

        Ok(SyncStaleTestRunsResponse { finished, active })
    }
}
