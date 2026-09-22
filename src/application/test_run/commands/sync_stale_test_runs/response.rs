use crate::domain::test_run::entities::test_run::TestRun;

pub struct SyncStaleTestRunsResponse {
    /// Прогоны, которые завершились именно сейчас: по ним уходит сообщение в чат
    pub finished: Vec<TestRun>,
}
