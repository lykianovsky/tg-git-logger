use crate::domain::test_run::ports::test_runner::TestRunProgress;

pub struct GetTestRunProgressResponse {
    /// Пусто, если CI ещё ничего не сообщает о ходе прогона
    pub progress: Option<TestRunProgress>,
}
