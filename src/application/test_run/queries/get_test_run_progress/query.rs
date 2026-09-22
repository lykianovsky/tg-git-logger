use crate::domain::test_run::entities::test_run::TestRun;

/// Ход идущего прогона: карточка показывает по нему полосу прогресса
pub struct GetTestRunProgressQuery {
    pub run: TestRun,
}
