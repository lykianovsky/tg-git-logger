use crate::domain::test_run::entities::test_run::TestRun;

pub struct SyncStaleTestRunsResponse {
    /// Прогоны, которые завершились именно сейчас: по ним уходит итог
    pub finished: Vec<TestRun>,
    /// Прогоны, которые ещё идут: по ним обновляется карточка
    pub active: Vec<TestRun>,
}
