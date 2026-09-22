use crate::domain::test_run::entities::test_suite::TestSuite;

pub struct GetTestSuiteResponse {
    /// Пусто, когда тесты к репозиторию не подключены
    pub suite: Option<TestSuite>,
}
