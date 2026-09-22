use crate::domain::test_run::entities::test_run::TestRun;

pub struct RerunFailedTestsResponse {
    pub run: TestRun,
    /// Сколько тестов ушло в перезапуск
    pub failures_count: usize,
}
