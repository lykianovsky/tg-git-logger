use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Текущие настройки тестов репозитория: что показывает экран настройки
pub struct GetTestSuiteQuery {
    pub repository_id: RepositoryId,
}
