use crate::domain::test_run::entities::test_run::TestRun;

pub struct GetLastTestRunResponse {
    /// Прогонов ещё не было — карточка предлагает запустить первый
    pub run: Option<TestRun>,
    /// Тесты к репозиторию не подключены — карточка предлагает подключить
    pub is_configured: bool,
}
